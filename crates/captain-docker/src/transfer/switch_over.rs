//! Switch-over mode: checks that the switch-over is safe, stops an item in the
//! source, copies its volumes again, starts it in the target, and checks it. The
//! source containers are stopped, never removed, and a roll back starts them again.
//! See docs/adr/0009-migration.md, "Switch-over mode".

mod check;
mod inspect;
mod preflight;
mod start;

use std::time::Instant;

use bollard::Docker;
use bollard::query_parameters::StopContainerOptionsBuilder;
use captain_core::EngineError;
use captain_core::migration::{MigrationItem, SwitchOverPlan, SwitchOverStep, TransferEvent};

use super::compose::Project;
use super::progress::{self, Events, Outcome};
use super::source::SourceEngine;
use super::volume::{Existing, copy_volume};
use crate::{ComposeCli, mapping};

pub use start::Start;

/// One switch-over and the names it touches.
pub struct Job<'a> {
    /// Containers to stop in the source. A roll back starts them again.
    pub stop: Vec<String>,
    /// Pairs of source and target volume names. The assistant keeps names; tests
    /// copy into new names on one engine.
    pub volumes: Vec<(String, String)>,
    pub start: Start<'a>,
    /// True if the target runs on this computer, so the check can connect to its
    /// published ports on 127.0.0.1.
    pub local: bool,
}

impl<'a> Job<'a> {
    /// A switch-over of the source container `name` into `target_name`.
    pub fn container(
        name: &'a str,
        target_name: &'a str,
        volumes: Vec<(String, String)>,
        local: bool,
    ) -> Self {
        Self {
            stop: vec![name.to_string()],
            volumes,
            start: Start::Container {
                id: name,
                target_name,
                snapshot: false,
            },
            local,
        }
    }

    /// The job for a scanned item, with every name kept. `None` if the item cannot
    /// switch over.
    pub fn for_item(
        item: &'a MigrationItem,
        snapshot: bool,
        cli: Result<ComposeCli, String>,
        local: bool,
    ) -> Option<Self> {
        let plan = SwitchOverPlan::for_item(item)?;
        let start = match item {
            MigrationItem::Container { id, name, .. } => Start::Container {
                id,
                target_name: name,
                snapshot,
            },
            MigrationItem::ComposeProject {
                name,
                working_dir,
                config_files,
                files_exist,
                containers,
                ..
            } => Start::Project {
                project: Project {
                    name,
                    working_dir: working_dir.as_deref(),
                    config_files,
                    files_exist: *files_exist,
                    containers,
                },
                services: plan.services,
                running: plan.stop.clone(),
                cli,
            },
            _ => return None,
        };
        Some(Self {
            volumes: plan
                .volumes
                .iter()
                .map(|v| (v.clone(), v.clone()))
                .collect(),
            stop: plan.stop,
            start,
            local,
        })
    }
}

/// Runs the switch-over steps in order and reports each one. Nothing is stopped
/// until the checks in [`preflight`] pass. Once the source is stopped, a closed
/// event stream no longer stops the work.
pub async fn switch_over(
    source: &SourceEngine,
    target: &Docker,
    job: Job<'_>,
    events: &Events,
) -> Result<Outcome, EngineError> {
    preflight::check(source, target, &job).await?;
    if progress::is_cancelled(events) {
        return Err(progress::cancelled());
    }
    let events = &progress::uncancellable(events);
    let began = Instant::now();
    enter(events, SwitchOverStep::StopSource);
    for name in &job.stop {
        stop_source(source, name).await?;
    }
    enter(events, SwitchOverStep::ResyncVolumes);
    for (from, to) in &job.volumes {
        copy_volume(source, target, (from, to), Existing::Replace, events).await?;
    }
    enter(events, SwitchOverStep::StartTarget);
    let started = start::start(source, target, &job.start, events).await?;
    enter(events, SwitchOverStep::Verify);
    check::verify(target, &started, job.local).await?;
    progress::send(events, TransferEvent::Downtime(began.elapsed()));
    enter(events, SwitchOverStep::Done);
    let note = "Runs here now. The original is stopped in the old engine, not deleted.";
    progress::send(events, TransferEvent::Note(note.into()));
    Ok(Outcome::Copied)
}

/// Undoes a switch-over: stops the item's containers in the target, then starts
/// the originals in the source. A copy that is missing or already stopped counts as
/// stopped, so a switch-over that failed before the start can roll back too. It
/// tries every step and reports the errors at the end. Nothing is removed.
pub async fn roll_back(
    source: &SourceEngine,
    target: &Docker,
    job: &Job<'_>,
) -> Result<(), EngineError> {
    let mut errors = Vec::new();
    let copies = match &job.start {
        Start::Container { target_name, .. } => Ok(vec![target_name.to_string()]),
        Start::Project {
            project, services, ..
        } => start::project_containers(target, project.name, services).await,
    };
    match copies {
        Ok(copies) => {
            for name in copies {
                errors.extend(stop_copy(target, &name).await.err());
            }
        }
        Err(error) => errors.push(error),
    }
    for name in &job.stop {
        errors.extend(source.start_container(name).await.err());
    }
    match errors.len() {
        0 => Ok(()),
        1 => Err(errors.remove(0)),
        _ => Err(EngineError::Api(
            errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; "),
        )),
    }
}

/// Stops the copy `name` in the target, unless it is missing or not running.
async fn stop_copy(target: &Docker, name: &str) -> Result<(), EngineError> {
    let Some(inspect) = mapping::found(target.inspect_container(name, None).await)? else {
        return Ok(());
    };
    if !inspect::is_running(&inspect) {
        return Ok(());
    }
    let timeout = inspect::stop_timeout(&inspect);
    let options = StopContainerOptionsBuilder::default().t(timeout).build();
    mapping::found(target.stop_container(name, Some(options)).await).map(|_| ())
}

fn enter(events: &Events, step: SwitchOverStep) {
    progress::send(events, TransferEvent::SwitchOver(step));
}

/// Stops one source container with its own stop timeout, or a generous default,
/// so a database shuts down cleanly. A container that already stopped is left.
async fn stop_source(source: &SourceEngine, name: &str) -> Result<(), EngineError> {
    let inspect = source.inspect_container(name).await?;
    if !inspect::is_running(&inspect) {
        return Ok(());
    }
    source
        .stop_container(name, inspect::stop_timeout(&inspect))
        .await
}

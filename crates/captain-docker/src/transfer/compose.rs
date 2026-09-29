//! Recreates a Compose project in the target: `docker compose up -d` from its files
//! when they exist here, or else container by container from the inspect data.

use std::collections::HashMap;

use bollard::Docker;
use bollard::query_parameters::ListContainersOptionsBuilder;
use captain_core::EngineError;
use captain_core::migration::TransferEvent;
use captain_core::model::ComposeProject;

use super::container::copy_container;
use super::owner;
use super::progress::{self, Events, Outcome};
use super::source::SourceEngine;
use crate::{ComposeCli, mapping};

/// The label with a container's Compose project name.
pub const PROJECT_LABEL: &str = "com.docker.compose.project";
/// The folder Compose ran a project from.
const WORKING_DIR_LABEL: &str = "com.docker.compose.project.working_dir";

/// A project as the plan lists it.
#[derive(Debug, Clone)]
pub struct Project<'a> {
    pub name: &'a str,
    pub working_dir: Option<&'a str>,
    pub config_files: &'a [String],
    pub files_exist: bool,
    /// Names of the project's containers in the source.
    pub containers: &'a [String],
}

/// Runs `up -d` with `cli` against the target when the files exist, and falls back
/// to recreating the containers one by one when they or the CLI are missing.
pub async fn copy_project(
    source: &SourceEngine,
    target: &Docker,
    cli: Result<ComposeCli, String>,
    project: Project<'_>,
    events: &Events,
) -> Result<Outcome, EngineError> {
    let reason = match (&cli, project.files_exist) {
        (Ok(cli), true) => {
            let compose = ComposeProject {
                name: project.name.into(),
                working_dir: project.working_dir.map(Into::into),
                config_files: project.config_files.to_vec(),
                services: Vec::new(),
            };
            let origin = owner::origin(source).await?;
            if foreign_project(target, &project, &origin).await? {
                let reason = format!(
                    "A Compose project named {} is already in this engine, and Captain did \
                     not copy it from the old engine. Captain left it as it is.",
                    project.name
                );
                return Ok(Outcome::Skipped(reason));
            }
            let labels = owner::mark(None, &origin);
            // A stop from the user drops the command, so no containers start later.
            progress::until_cancelled(events, cli.up_labeled(&compose, &[], labels)).await?;
            let note = "Started with docker compose up -d from its files.";
            progress::send(events, TransferEvent::Note(note.into()));
            return Ok(Outcome::Copied);
        }
        (_, false) => "its Compose files are not on this computer".to_string(),
        (Err(error), true) => error.clone(),
    };
    let mut skipped = 0;
    for (ix, name) in project.containers.iter().enumerate() {
        if progress::is_cancelled(events) {
            return Err(progress::cancelled());
        }
        let total = project.containers.len() as u64;
        progress::send(
            events,
            TransferEvent::Progress {
                done: ix as u64,
                total,
            },
        );
        if let Outcome::Skipped(_) =
            copy_container(source, target, name, None, false, events).await?
        {
            skipped += 1;
        }
    }
    if skipped == project.containers.len() {
        let reason = "Its containers are already in the target.";
        return Ok(Outcome::Skipped(reason.into()));
    }
    let note = format!("Recreated container by container, because {reason}.");
    progress::send(events, TransferEvent::Note(note));
    Ok(Outcome::Copied)
}

/// True if the target has containers of a project named like `project` that
/// Captain did not copy there from `origin`, or that Compose ran from another
/// folder. `docker compose up` would take them over.
pub async fn foreign_project(
    target: &Docker,
    project: &Project<'_>,
    origin: &str,
) -> Result<bool, EngineError> {
    let label = format!("{PROJECT_LABEL}={}", project.name);
    let filters = HashMap::from([("label", vec![label.as_str()])]);
    let options = ListContainersOptionsBuilder::default()
        .all(true)
        .filters(&filters)
        .build();
    let listed = target.list_containers(Some(options)).await;
    Ok(listed.map_err(mapping::engine_error)?.into_iter().any(|c| {
        let labels = c.labels.as_ref();
        let dir = labels.and_then(|l| l.get(WORKING_DIR_LABEL));
        dir.map(String::as_str) != project.working_dir || !owner::is_copy_from(labels, origin)
    }))
}

//! Recreates a Compose project in the target: `docker compose up -d` from its files
//! when they exist here, or else container by container from the inspect data.

use bollard::Docker;
use captain_core::EngineError;
use captain_core::migration::TransferEvent;
use captain_core::model::ComposeProject;

use super::container::copy_container;
use super::owner;
use super::progress::{self, Events, Outcome};
use super::source::SourceEngine;
use crate::ComposeCli;

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
            let labels = owner::mark(None, &owner::origin(source).await?);
            cli.up_labeled(&compose, &[], labels).await?;
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

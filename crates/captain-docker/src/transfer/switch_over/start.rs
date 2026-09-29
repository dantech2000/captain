//! Starts a switched item in the target: a Compose project with `up -d` for only the
//! services that ran in the source, or a recreated container.

use std::collections::HashMap;

use bollard::Docker;
use bollard::query_parameters::ListContainersOptionsBuilder;
use captain_core::EngineError;
use captain_core::model::ComposeProject;

use super::super::compose::{PROJECT_LABEL, Project};
use super::super::container::copy_container;
use super::super::owner;
use super::super::progress::Events;
use super::super::source::SourceEngine;
use crate::{ComposeCli, mapping};

const SERVICE_LABEL: &str = "com.docker.compose.service";
/// Set on containers from `docker compose run`, which are not part of the service.
const ONE_OFF_LABEL: &str = "com.docker.compose.oneoff";

/// How the item starts in the target.
pub enum Start<'a> {
    /// A Compose project. `services` ran in the source, and `running` names their
    /// containers.
    Project {
        project: Project<'a>,
        services: Vec<String>,
        running: Vec<String>,
        cli: Result<ComposeCli, String>,
    },
    /// A standalone container, recreated as `target_name`.
    Container {
        id: &'a str,
        target_name: &'a str,
        snapshot: bool,
    },
}

/// Starts the item and returns the names of its running containers in the target.
pub async fn start(
    source: &SourceEngine,
    target: &Docker,
    start: &Start<'_>,
    events: &Events,
) -> Result<Vec<String>, EngineError> {
    match start {
        Start::Container {
            id,
            target_name,
            snapshot,
        } => {
            // The source is stopped now, so the copy is created and not started.
            copy_container(source, target, id, Some(target_name), *snapshot, events).await?;
            start_all(target, &[target_name.to_string()]).await?;
            Ok(vec![target_name.to_string()])
        }
        Start::Project {
            project,
            services,
            cli: Ok(cli),
            ..
        } if project.files_exist => {
            let compose = ComposeProject {
                name: project.name.into(),
                working_dir: project.working_dir.map(Into::into),
                config_files: project.config_files.to_vec(),
                services: Vec::new(),
            };
            // Labeled, so a later switch-over knows that Captain created them. It
            // runs to the end: the source is stopped already.
            let labels = owner::mark(None, &owner::origin(source).await?);
            cli.up_labeled_to_end(&compose, services, labels).await?;
            project_containers(target, project.name, services).await
        }
        Start::Project {
            project, running, ..
        } => {
            for name in project.containers {
                copy_container(source, target, name, None, false, events).await?;
            }
            start_all(target, running).await?;
            Ok(running.clone())
        }
    }
}

async fn start_all(target: &Docker, names: &[String]) -> Result<(), EngineError> {
    for name in names {
        let started = target.start_container(name, None).await;
        started.map_err(mapping::engine_error)?;
    }
    Ok(())
}

/// The names of the containers of `services` in the project `project` in `docker`.
pub async fn project_containers(
    docker: &Docker,
    project: &str,
    services: &[String],
) -> Result<Vec<String>, EngineError> {
    let label = format!("{PROJECT_LABEL}={project}");
    let filters = HashMap::from([("label", vec![label.as_str()])]);
    let options = ListContainersOptionsBuilder::default()
        .all(true)
        .filters(&filters)
        .build();
    let containers = docker.list_containers(Some(options)).await;
    Ok(containers
        .map_err(mapping::engine_error)?
        .into_iter()
        .filter(|c| {
            let labels = c.labels.as_ref();
            let service = labels.and_then(|l| l.get(SERVICE_LABEL));
            let one_off = labels.and_then(|l| l.get(ONE_OFF_LABEL));
            service.is_some_and(|s| services.contains(s)) && one_off.is_none_or(|v| v != "True")
        })
        .filter_map(|c| {
            c.names?
                .first()
                .map(|n| n.trim_start_matches('/').to_string())
        })
        .collect())
}

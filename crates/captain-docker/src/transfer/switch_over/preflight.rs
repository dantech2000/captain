//! Checks that run before a switch-over stops anything in the source. Each one
//! refuses the switch-over, with the reason, when the stop or the copy could lose
//! data. See docs/adr/0009-migration.md, "Switch-over mode".

use std::collections::HashMap;

use bollard::Docker;
use bollard::models::{ContainerInspectResponse, ContainerSummary};
use bollard::query_parameters::ListContainersOptionsBuilder;
use captain_core::EngineError;

use super::super::compose::Project;
use super::super::owner;
use super::super::source::SourceEngine;
use super::Job;
use super::start::{PROJECT_LABEL, Start};
use crate::mapping;

/// The folder Compose ran a project from.
const WORKING_DIR_LABEL: &str = "com.docker.compose.project.working_dir";

/// Runs every check for `job`, in the source first, then in the target.
pub async fn check(
    source: &SourceEngine,
    target: &Docker,
    job: &Job<'_>,
) -> Result<(), EngineError> {
    for name in &job.stop {
        auto_remove(name, &source.inspect_container(name).await?)?;
    }
    let volumes: Vec<&str> = job.volumes.iter().map(|(from, _)| from.as_str()).collect();
    other_writers(&volumes, &job.stop, &source.list_containers().await?)?;
    let origin = owner::origin(source).await?;
    for (_, to) in &job.volumes {
        let volume = mapping::found(target.inspect_volume(to).await)?;
        if volume.is_some_and(|v| !owner::is_copy_from(Some(&v.labels), &origin)) {
            return Err(not_a_copy(&format!("The volume {to}")));
        }
    }
    let names: Vec<&str> = match &job.start {
        Start::Container { target_name, .. } => vec![target_name],
        Start::Project {
            project,
            cli: Ok(_),
            ..
        } if project.files_exist => {
            return same_project(target, project, &origin).await;
        }
        Start::Project { project, .. } => project.containers.iter().map(String::as_str).collect(),
    };
    for name in names {
        let found = mapping::found(target.inspect_container(name, None).await)?;
        let labels = found
            .as_ref()
            .and_then(|c| c.config.as_ref()?.labels.as_ref());
        if found.is_some() && !owner::is_copy_from(labels, &origin) {
            return Err(not_a_copy(&format!("The container {name}")));
        }
    }
    Ok(())
}

/// Refuses a container started with `--rm`: the engine deletes it when it stops,
/// so a stop would lose it and a roll back could not start it again.
pub fn auto_remove(name: &str, inspect: &ContainerInspectResponse) -> Result<(), EngineError> {
    let removes = inspect.host_config.as_ref().and_then(|h| h.auto_remove);
    if removes != Some(true) {
        return Ok(());
    }
    Err(EngineError::Api(format!(
        "{name} was started with --rm, so the old engine deletes it when it stops. \
         Captain cannot switch it over safely. Copy it without switch-over."
    )))
}

/// Refuses while a running container that the switch-over does not stop mounts one
/// of `volumes` for writing. It would keep writing while Captain copies.
pub fn other_writers(
    volumes: &[&str],
    stop: &[String],
    containers: &[ContainerSummary],
) -> Result<(), EngineError> {
    for volume in volumes {
        let writers: Vec<String> = containers
            .iter()
            .filter(|c| c.state.is_some_and(|s| s.as_ref() == "running"))
            .filter(|c| {
                c.mounts
                    .iter()
                    .flatten()
                    .any(|m| m.name.as_deref() == Some(*volume) && m.rw != Some(false))
            })
            .filter_map(|c| {
                let name = c.names.as_ref()?.first()?.trim_start_matches('/');
                (!stop.iter().any(|s| s == name)).then(|| name.to_string())
            })
            .collect();
        if !writers.is_empty() {
            return Err(EngineError::Api(format!(
                "{} also runs in the old engine and writes to {volume}. Stop it there, \
                 or switch over the item it belongs to, then retry.",
                writers.join(", ")
            )));
        }
    }
    Ok(())
}

/// Refuses when the target has containers of a project named like `project` that
/// Captain did not copy there from `origin`, or that Compose ran from another
/// folder. `docker compose up` would take them over.
async fn same_project(
    target: &Docker,
    project: &Project<'_>,
    origin: &str,
) -> Result<(), EngineError> {
    let label = format!("{PROJECT_LABEL}={}", project.name);
    let filters = HashMap::from([("label", vec![label.as_str()])]);
    let options = ListContainersOptionsBuilder::default()
        .all(true)
        .filters(&filters)
        .build();
    let listed = target.list_containers(Some(options)).await;
    let other = listed.map_err(mapping::engine_error)?.into_iter().any(|c| {
        let labels = c.labels.as_ref();
        let dir = labels.and_then(|l| l.get(WORKING_DIR_LABEL));
        dir.map(String::as_str) != project.working_dir || !owner::is_copy_from(labels, origin)
    });
    if other {
        let what = format!("A Compose project named {}", project.name);
        return Err(not_a_copy(&what));
    }
    Ok(())
}

/// The refusal for something in the target that Captain did not copy there.
fn not_a_copy(what: &str) -> EngineError {
    EngineError::Api(format!(
        "{what} is already in this engine, and Captain did not copy it from the old \
         engine. Captain will not replace or start it. Rename or remove it here, then retry."
    ))
}

#[cfg(test)]
mod tests;

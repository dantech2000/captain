//! `docker run -d`: create a container from a spec, then start it.

use bollard::Docker;
use bollard::query_parameters::CreateContainerOptionsBuilder;
use captain_core::EngineError;
use captain_core::model::RunSpec;

use crate::mapping;

/// Creates and starts a container. Returns its ID. If the start fails, the created
/// container stays, as it does with the Docker CLI.
pub async fn run_container(docker: &Docker, spec: RunSpec) -> Result<String, EngineError> {
    let mut options = CreateContainerOptionsBuilder::default();
    if let Some(name) = &spec.name {
        options = options.name(name);
    }
    let created = docker
        .create_container(Some(options.build()), mapping::run_body(&spec))
        .await
        .map_err(mapping::engine_error)?;
    for warning in &created.warnings {
        tracing::warn!(%warning, id = %created.id, "the engine warned about a new container");
    }
    docker
        .start_container(&created.id, None)
        .await
        .map_err(mapping::engine_error)?;
    Ok(created.id)
}

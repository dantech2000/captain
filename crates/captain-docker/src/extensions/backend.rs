//! The backend: a Compose project named `captain-ext-<id>`, run with the `docker
//! compose` CLI, with a proxy that publishes its socket on `127.0.0.1`.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use bollard::Docker;
use bollard::query_parameters::{ListContainersOptionsBuilder, ListVolumesOptionsBuilder};
use captain_core::EngineError;
use captain_core::extension::{
    Backend, COMPOSE_FILE, InstalledExtension, PROXY_PORT, PROXY_SERVICE, image_project,
    project_name, with_guest_services,
};
use serde_json::Value;

use super::files;
use super::manager::Context;
use crate::compose::error_message;
use crate::mapping;
use crate::process::output_within;

const PROJECT_LABEL: &str = "com.docker.compose.project";
const SERVICE_LABEL: &str = "com.docker.compose.service";
/// Docker Desktop sets this for extension Compose files.
const IMAGE_VARIABLE: &str = "DESKTOP_PLUGIN_IMAGE";
/// How long one `compose` step may take. `up` may pull the backend's images.
const COMPOSE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// Writes the project file and runs `compose up -d`.
pub async fn up(
    context: &Context,
    extension: &InstalledExtension,
    backend: Backend,
) -> Result<(), EngineError> {
    let id = &extension.id;
    let dir = context.paths.compose_dir(id);
    std::fs::create_dir_all(&dir).map_err(files::io_error)?;
    let socket = extension.metadata.backend_socket();
    let project = match backend {
        Backend::Image(image) => image_project(id, &image, socket),
        Backend::Compose(file) => {
            let name = captain_core::extension::binary_name(&file).to_string();
            let mut command = compose(context, extension.pinned_image(), &dir)?;
            command.args(["-f", &name, "config", "--format", "json"]);
            let config: Value = serde_json::from_str(&run(command).await?).map_err(|error| {
                EngineError::Api(format!("cannot read the Compose file: {error}"))
            })?;
            with_guest_services(config, id, socket)
        }
    };
    let text = serde_json::to_string_pretty(&project).unwrap_or_default();
    std::fs::write(dir.join(COMPOSE_FILE), text).map_err(files::io_error)?;
    let mut command = compose(context, extension.pinned_image(), &dir)?;
    command.args([
        "-p",
        &project_name(id),
        "-f",
        COMPOSE_FILE,
        "up",
        "-d",
        "--quiet-pull",
    ]);
    run(command).await.map(drop)
}

/// Stops the project and removes its containers, and its volumes too with
/// `volumes`. Compose finds them by the project label.
pub async fn down(context: &Context, id: &str, volumes: bool) -> Result<(), EngineError> {
    let mut command = context.docker_command()?;
    command.args(["compose", "-p", &project_name(id)]);
    // Without `-f`, Compose looks for a file in the working folder and its parents,
    // so it runs where `up` ran, on Captain's file, or where there is none.
    let dir = context.paths.compose_dir(id);
    if dir.join(COMPOSE_FILE).is_file() {
        command.current_dir(&dir).args(["-f", COMPOSE_FILE]);
    } else {
        command.current_dir(std::env::temp_dir());
    }
    command.args(["down", "--remove-orphans"]);
    if volumes {
        command.arg("--volumes");
    }
    run(command).await.map(drop)
}

/// Whether the engine has a container or a volume of the extension's project.
pub async fn exists(docker: &Docker, id: &str) -> Result<bool, EngineError> {
    let label = format!("{PROJECT_LABEL}={}", project_name(id));
    let filters = HashMap::from([("label", vec![label.as_str()])]);
    let options = ListContainersOptionsBuilder::default()
        .all(true)
        .filters(&filters)
        .build();
    let containers = docker
        .list_containers(Some(options))
        .await
        .map_err(mapping::engine_error)?;
    let options = ListVolumesOptionsBuilder::default()
        .filters(&filters)
        .build();
    let volumes = docker
        .list_volumes(Some(options))
        .await
        .map_err(mapping::engine_error)?;
    Ok(!containers.is_empty() || volumes.volumes.is_some_and(|v| !v.is_empty()))
}

fn compose(context: &Context, image: &str, dir: &Path) -> Result<Command, EngineError> {
    let mut command = context.docker_command()?;
    command
        .arg("compose")
        .current_dir(dir)
        .env(IMAGE_VARIABLE, image);
    Ok(command)
}

/// Runs `command` on a blocking thread. Returns stdout, or fails with the error line.
/// A command that runs past [`COMPOSE_TIMEOUT`] is killed.
async fn run(command: Command) -> Result<String, EngineError> {
    let output = tokio::task::spawn_blocking(move || output_within(command, COMPOSE_TIMEOUT))
        .await
        .map_err(|error| EngineError::Api(error.to_string()))?
        .map_err(|error| EngineError::Api(format!("docker compose: {error}")))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(EngineError::Api(error_message(&stderr).unwrap_or_else(
        || format!("docker compose exited with {}", output.status),
    )))
}

/// A running container of an extension's project.
struct Service {
    name: String,
    container: String,
    /// Container port and host port pairs.
    ports: Vec<(u16, u16)>,
}

/// The running containers of the extension's project, sorted by service name.
async fn services(docker: &Docker, id: &str) -> Result<Vec<Service>, EngineError> {
    let label = format!("{PROJECT_LABEL}={}", project_name(id));
    let filters = HashMap::from([("label", vec![label.as_str()])]);
    let options = ListContainersOptionsBuilder::default()
        .filters(&filters)
        .build();
    let containers = docker
        .list_containers(Some(options))
        .await
        .map_err(mapping::engine_error)?;
    let mut services: Vec<Service> = containers
        .into_iter()
        .filter_map(|container| {
            let name = container.labels?.get(SERVICE_LABEL)?.clone();
            let ports = container
                .ports
                .unwrap_or_default()
                .into_iter()
                .filter_map(|port| Some((port.private_port, port.public_port?)))
                .collect();
            Some(Service {
                name,
                container: container.id?,
                ports,
            })
        })
        .collect();
    services.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(services)
}

fn not_running() -> EngineError {
    EngineError::Api("the extension's backend is not running".into())
}

/// The host port of the proxy, on `127.0.0.1`.
pub async fn proxy_port(docker: &Docker, id: &str) -> Result<u16, EngineError> {
    services(docker, id)
        .await?
        .into_iter()
        .filter(|service| service.name == PROXY_SERVICE)
        .flat_map(|service| service.ports)
        .find_map(|(private, public)| (private == PROXY_PORT).then_some(public))
        .ok_or_else(not_running)
}

/// The container that `extension.vm.cli.exec` runs in: the first backend service.
pub async fn backend_container(docker: &Docker, id: &str) -> Result<String, EngineError> {
    services(docker, id)
        .await?
        .into_iter()
        .find(|service| service.name != PROXY_SERVICE)
        .map(|service| service.container)
        .ok_or_else(not_running)
}

//! Helper containers: tiny busybox containers that mount a volume so Captain can
//! read, write, and measure it. Every helper is named `captain-migrate-*`, carries
//! [`HELPER_LABEL`], and is removed when its job ends.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use bollard::Docker;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, ListContainersOptionsBuilder,
    LogsOptionsBuilder, RemoveContainerOptionsBuilder,
};
use captain_core::EngineError;
use captain_core::migration::HELPER_PREFIX;
use futures::StreamExt;

use crate::mapping;

/// The helper image. Captain pulls it only when the engine does not have it.
pub const HELPER_IMAGE: &str = "busybox:latest";
/// The label on every helper container.
pub const HELPER_LABEL: &str = "io.captain.migrate";

/// A volume for a helper to mount at `/v`.
#[derive(Debug, Clone, Copy)]
pub struct Mount<'a> {
    pub volume: &'a str,
    pub read_only: bool,
}

/// A unique `captain-migrate-<purpose>-<id>` name.
pub fn helper_name(purpose: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{HELPER_PREFIX}-{purpose}-{nanos:08x}{count}")
}

/// Pulls [`HELPER_IMAGE`] if the engine does not have it. Returns true if it pulled,
/// so the caller can remove it again. An image that exists is never pulled, so a
/// tag the user has stays where it is.
pub async fn ensure_image(docker: &Docker) -> Result<bool, EngineError> {
    if docker.inspect_image(HELPER_IMAGE).await.is_ok() {
        return Ok(false);
    }
    let (name, tag) = HELPER_IMAGE
        .split_once(':')
        .unwrap_or((HELPER_IMAGE, "latest"));
    let options = CreateImageOptionsBuilder::default()
        .from_image(name)
        .tag(tag)
        .build();
    let mut pull = docker.create_image(Some(options), None, None);
    while let Some(message) = pull.next().await {
        message.map_err(mapping::engine_error)?;
    }
    Ok(true)
}

/// Creates a helper that is not started, and returns its ID.
pub async fn create(
    docker: &Docker,
    purpose: &str,
    mount: Option<Mount<'_>>,
    cmd: &[&str],
) -> Result<String, EngineError> {
    let binds = mount.map(|m| {
        let mode = if m.read_only { ":ro" } else { "" };
        vec![format!("{}:/v{mode}", m.volume)]
    });
    let body = ContainerCreateBody {
        image: Some(HELPER_IMAGE.into()),
        cmd: Some(cmd.iter().map(ToString::to_string).collect()),
        labels: Some(HashMap::from([(HELPER_LABEL.into(), "helper".into())])),
        network_disabled: Some(true),
        host_config: Some(HostConfig {
            binds,
            network_mode: Some("none".into()),
            ..HostConfig::default()
        }),
        ..ContainerCreateBody::default()
    };
    let options = CreateContainerOptionsBuilder::default()
        .name(&helper_name(purpose))
        .build();
    docker
        .create_container(Some(options), body)
        .await
        .map(|created| created.id)
        .map_err(mapping::engine_error)
}

/// Runs `script` with `sh -c` in a new helper, waits for it, removes it, and returns
/// what it printed. A script that exits with an error fails with its output.
pub async fn run_script(
    docker: &Docker,
    purpose: &str,
    mount: Option<Mount<'_>>,
    script: &str,
) -> Result<String, EngineError> {
    let id = create(docker, purpose, mount, &["sh", "-c", script]).await?;
    let result = run_to_end(docker, &id).await;
    remove(docker, &id).await;
    result
}

async fn run_to_end(docker: &Docker, id: &str) -> Result<String, EngineError> {
    docker
        .start_container(id, None)
        .await
        .map_err(mapping::engine_error)?;
    let mut wait = docker.wait_container(id, None);
    let mut failed = None;
    while let Some(status) = wait.next().await {
        if let Err(error) = status {
            failed = Some(error.to_string());
        }
    }
    let options = LogsOptionsBuilder::default()
        .stdout(true)
        .stderr(true)
        .build();
    let mut logs = docker.logs(id, Some(options));
    let mut output = String::new();
    while let Some(line) = logs.next().await {
        let line = line.map_err(mapping::engine_error)?;
        output.push_str(&String::from_utf8_lossy(&line.into_bytes()));
    }
    match failed {
        None => Ok(output),
        Some(error) => Err(EngineError::Api(format!("{error}: {}", output.trim()))),
    }
}

/// Removes a helper and its anonymous volumes. Failures are logged, not returned,
/// because this runs during cleanup.
pub async fn remove(docker: &Docker, id: &str) {
    let options = RemoveContainerOptionsBuilder::default()
        .force(true)
        .v(true)
        .build();
    if let Err(error) = docker.remove_container(id, Some(options)).await {
        tracing::warn!(%error, id, "cannot remove a migration helper");
    }
}

/// The IDs of helpers left behind, for example by a crash: containers that have
/// both the helper label and the helper name.
pub async fn leftovers(docker: &Docker) -> Result<Vec<String>, EngineError> {
    let filters = HashMap::from([("label", vec![HELPER_LABEL])]);
    let options = ListContainersOptionsBuilder::default()
        .all(true)
        .filters(&filters)
        .build();
    let containers = docker
        .list_containers(Some(options))
        .await
        .map_err(mapping::engine_error)?;
    let prefix = format!("/{HELPER_PREFIX}-");
    Ok(containers
        .into_iter()
        .filter(|c| {
            c.names
                .iter()
                .flatten()
                .any(|name| name.starts_with(&prefix))
        })
        .filter_map(|c| c.id)
        .collect())
}

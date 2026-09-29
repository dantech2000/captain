//! Scans an image with Trivy, run as a container on the engine, as Rancher Desktop
//! does. See docs/features/0019-image-build-push-scan.md.

use std::time::{SystemTime, UNIX_EPOCH};

use bollard::Docker;
use bollard::container::LogOutput;
use bollard::errors::Error;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, LogsOptionsBuilder,
    RemoveContainerOptionsBuilder,
};
use captain_core::EngineError;
use captain_core::model::{ScanProgress, ScanReport, parse_trivy_report, trivy_status};
use futures::StreamExt;
use futures::channel::mpsc::UnboundedSender;

use crate::mapping;

const TRIVY_REPO: &str = "aquasec/trivy";
const TRIVY_IMAGE: &str = "aquasec/trivy:latest";
/// Keeps Trivy's vulnerability database between scans.
const CACHE_VOLUME: &str = "captain-trivy-cache";
/// The engine socket inside the engine's machine.
const ENGINE_SOCKET: &str = "/var/run/docker.sock";

type Sender = UnboundedSender<Result<ScanProgress, EngineError>>;

/// Scans `reference` and sends status lines, then the report or one error.
pub async fn scan(docker: Docker, reference: String, tx: Sender) {
    if let Err(error) = ensure_trivy(&docker, &tx).await {
        tx.unbounded_send(Err(error)).ok();
        return;
    }
    let id = match create(&docker, &reference).await {
        Ok(id) => id,
        Err(error) => {
            tx.unbounded_send(Err(error)).ok();
            return;
        }
    };
    let result = run(&docker, &id, &tx).await;
    let options = RemoveContainerOptionsBuilder::default().force(true).build();
    if let Err(error) = docker.remove_container(&id, Some(options)).await {
        tracing::warn!(%error, %id, "cannot remove the Trivy container");
    }
    if let Some(result) = result {
        tx.unbounded_send(result.map(ScanProgress::Report)).ok();
    }
}

/// Pulls the Trivy image the first time.
async fn ensure_trivy(docker: &Docker, tx: &Sender) -> Result<(), EngineError> {
    if docker.inspect_image(TRIVY_IMAGE).await.is_ok() {
        return Ok(());
    }
    let status = ScanProgress::Status(format!("Pulling {TRIVY_IMAGE}"));
    tx.unbounded_send(Ok(status)).ok();
    let options = CreateImageOptionsBuilder::default()
        .from_image(TRIVY_REPO)
        .tag("latest")
        .build();
    let mut pull = docker.create_image(Some(options), None, None);
    while let Some(message) = pull.next().await {
        message.map_err(mapping::pull_error)?;
    }
    Ok(())
}

async fn create(docker: &Docker, reference: &str) -> Result<String, EngineError> {
    let cmd = [
        "image",
        "--format",
        "json",
        "--scanners",
        "vuln",
        "--no-progress",
    ];
    let body = ContainerCreateBody {
        image: Some(TRIVY_IMAGE.into()),
        cmd: Some(
            cmd.iter()
                .map(ToString::to_string)
                .chain([reference.to_string()])
                .collect(),
        ),
        host_config: Some(HostConfig {
            binds: Some(vec![
                format!("{ENGINE_SOCKET}:{ENGINE_SOCKET}"),
                format!("{CACHE_VOLUME}:/root/.cache/"),
            ]),
            ..HostConfig::default()
        }),
        ..ContainerCreateBody::default()
    };
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    let options = CreateContainerOptionsBuilder::default()
        .name(&format!("captain-scan-{nanos:08x}"))
        .build();
    docker
        .create_container(Some(options), body)
        .await
        .map(|created| created.id)
        .map_err(mapping::engine_error)
}

/// Starts the container and follows its output to the end. `None` when the
/// receiver is gone.
async fn run(docker: &Docker, id: &str, tx: &Sender) -> Option<Result<ScanReport, EngineError>> {
    if let Err(error) = docker.start_container(id, None).await {
        return Some(Err(mapping::engine_error(error)));
    }
    let options = LogsOptionsBuilder::default()
        .follow(true)
        .stdout(true)
        .stderr(true)
        .build();
    let mut logs = docker.logs(id, Some(options));
    let mut report = Vec::new();
    let mut stderr = String::new();
    let mut last_error = None;
    while let Some(chunk) = logs.next().await {
        match chunk {
            Ok(LogOutput::StdOut { message }) => report.extend_from_slice(&message),
            Ok(LogOutput::StdErr { message }) => {
                stderr.push_str(&String::from_utf8_lossy(&message));
                while let Some(end) = stderr.find('\n') {
                    let line: String = stderr.drain(..=end).collect();
                    let Some(status) = trivy_status(&line) else {
                        continue;
                    };
                    if line.contains("\tFATAL\t") || line.contains("\tERROR\t") {
                        last_error = Some(status.clone());
                    }
                    if tx.unbounded_send(Ok(ScanProgress::Status(status))).is_err() {
                        return None;
                    }
                }
            }
            Ok(_) => {}
            Err(error) => return Some(Err(mapping::engine_error(error))),
        }
    }
    let code = match docker.wait_container(id, None).next().await {
        Some(Ok(response)) => response.status_code,
        Some(Err(Error::DockerContainerWaitError { code, .. })) => code,
        Some(Err(error)) => return Some(Err(mapping::engine_error(error))),
        None => 0,
    };
    if code != 0 {
        let message = last_error.unwrap_or_else(|| format!("Trivy exited with code {code}"));
        return Some(Err(EngineError::Api(message)));
    }
    Some(parse_trivy_report(&String::from_utf8_lossy(&report)).map_err(EngineError::Api))
}

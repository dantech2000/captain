//! The check after a switch-over starts the target: each container must become
//! healthy, or keep running for a while when it has no health check, and each
//! published port must accept a connection on this computer.

use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use bollard::Docker;
use bollard::models::{ContainerInspectResponse, HealthStatusEnum};
use captain_core::EngineError;

use super::inspect::{exit_code, health, is_running, published_tcp_ports, started_at};
use crate::mapping;

/// How long a container with a health check may take to become healthy.
const HEALTH_TIMEOUT: Duration = Duration::from_secs(120);
/// How long a container without a health check must keep running.
const STEADY: Duration = Duration::from_secs(10);
/// How long a published port may take to accept a connection. The engine's VM
/// forwards a port a moment after the container starts.
const PORT_TIMEOUT: Duration = Duration::from_secs(30);
const POLL: Duration = Duration::from_millis(500);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(1);

/// Checks `containers` in the target. With `local`, the target runs on this
/// computer, so the check also connects to the published ports on 127.0.0.1.
pub async fn verify(
    target: &Docker,
    containers: &[String],
    local: bool,
) -> Result<(), EngineError> {
    let settled = containers.iter().map(|name| settle(target, name));
    futures::future::try_join_all(settled).await?;
    if !local {
        return Ok(());
    }
    for name in containers {
        for port in published_tcp_ports(&inspect(target, name).await?) {
            reach(name, port).await?;
        }
    }
    Ok(())
}

async fn inspect(target: &Docker, name: &str) -> Result<ContainerInspectResponse, EngineError> {
    let inspect = target.inspect_container(name, None).await;
    inspect.map_err(mapping::engine_error)
}

/// Waits until `name` is healthy, or has kept running for [`STEADY`] when it has no
/// health check.
async fn settle(target: &Docker, name: &str) -> Result<(), EngineError> {
    let first = inspect(target, name).await?;
    let started = started_at(&first).map(str::to_string);
    let checked = health(&first).is_some();
    let limit = if checked { HEALTH_TIMEOUT } else { STEADY };
    let begin = Instant::now();
    loop {
        let now = inspect(target, name).await?;
        if !is_running(&now) {
            return Err(EngineError::Api(format!(
                "{name} stopped in this engine with exit code {}.",
                exit_code(&now)
            )));
        }
        if started_at(&now) != started.as_deref() {
            return Err(EngineError::Api(format!(
                "{name} restarted in this engine. Check its logs."
            )));
        }
        let status = health(&now);
        if status == Some(HealthStatusEnum::HEALTHY) {
            return Ok(());
        }
        if begin.elapsed() >= limit {
            return match status {
                None => Ok(()),
                Some(status) => Err(EngineError::Api(format!(
                    "{name} is not healthy after {} s (it reports {status}).",
                    limit.as_secs()
                ))),
            };
        }
        tokio::time::sleep(POLL).await;
    }
}

/// Connects to `port` on 127.0.0.1 until it answers or [`PORT_TIMEOUT`] passes.
async fn reach(name: &str, port: u16) -> Result<(), EngineError> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let begin = Instant::now();
    loop {
        let connect = tokio::task::spawn_blocking(move || {
            TcpStream::connect_timeout(&address, CONNECT_TIMEOUT)
        });
        if matches!(connect.await, Ok(Ok(_))) {
            return Ok(());
        }
        if begin.elapsed() >= PORT_TIMEOUT {
            return Err(EngineError::Api(format!(
                "Nothing answers on localhost:{port} for {name}. Another program may hold the port."
            )));
        }
        tokio::time::sleep(POLL).await;
    }
}

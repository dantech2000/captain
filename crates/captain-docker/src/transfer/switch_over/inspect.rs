//! Reads what a switch-over needs from a container's inspect data. Pure, so tests
//! need no engine.

use bollard::models::{ContainerInspectResponse, HealthStatusEnum};

/// Seconds a stop waits before the engine kills the container, when the container
/// sets no stop timeout. The engine's own default is 10 s, which is short for a
/// database that flushes on shutdown.
pub const DEFAULT_STOP_TIMEOUT: i32 = 30;

/// The seconds to wait for a clean stop: the container's own `StopTimeout`, or
/// [`DEFAULT_STOP_TIMEOUT`].
pub fn stop_timeout(inspect: &ContainerInspectResponse) -> i32 {
    inspect
        .config
        .as_ref()
        .and_then(|config| config.stop_timeout)
        .filter(|seconds| *seconds > 0)
        .map_or(DEFAULT_STOP_TIMEOUT, |seconds| {
            i32::try_from(seconds).unwrap_or(i32::MAX)
        })
}

pub fn is_running(inspect: &ContainerInspectResponse) -> bool {
    inspect
        .state
        .as_ref()
        .and_then(|state| state.running)
        .unwrap_or(false)
}

/// When the container last started, to notice a restart.
pub fn started_at(inspect: &ContainerInspectResponse) -> Option<&str> {
    inspect.state.as_ref()?.started_at.as_deref()
}

pub fn exit_code(inspect: &ContainerInspectResponse) -> i64 {
    inspect
        .state
        .as_ref()
        .and_then(|state| state.exit_code)
        .unwrap_or(0)
}

/// The health status, or `None` when the container has no health check.
pub fn health(inspect: &ContainerInspectResponse) -> Option<HealthStatusEnum> {
    let status = inspect.state.as_ref()?.health.as_ref()?.status?;
    (!matches!(status, HealthStatusEnum::NONE | HealthStatusEnum::EMPTY)).then_some(status)
}

/// Host ports published for TCP on an address that `127.0.0.1` reaches: all
/// addresses, or the loopback. Sorted, without duplicates.
pub fn published_tcp_ports(inspect: &ContainerInspectResponse) -> Vec<u16> {
    let ports = inspect
        .network_settings
        .as_ref()
        .and_then(|settings| settings.ports.as_ref());
    let mut found: Vec<u16> = ports
        .into_iter()
        .flatten()
        .filter(|(port, _)| port.ends_with("/tcp"))
        .flat_map(|(_, bindings)| bindings.iter().flatten())
        .filter(|binding| {
            matches!(
                binding.host_ip.as_deref().unwrap_or(""),
                "" | "0.0.0.0" | "::" | "127.0.0.1"
            )
        })
        .filter_map(|binding| binding.host_port.as_deref()?.parse().ok())
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}

#[cfg(test)]
mod tests;

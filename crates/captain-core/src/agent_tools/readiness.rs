//! When `wait_for_healthy` may stop: every container runs and passes its health
//! check, or one stopped for good.

use schemars::JsonSchema;
use serde::Serialize;

use super::untrusted::plain;
use crate::model::{Container, ContainerState, Health};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct WaitReport {
    /// The container or project.
    pub target: String,
    /// True when every container runs and none fails or awaits its health check.
    pub ready: bool,
    /// Why the wait ended without `ready`, or what it still waits for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub waited_seconds: u64,
    /// `name: state` or `name: state, health`.
    pub containers: Vec<String>,
}

/// Where a wait stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Readiness {
    Ready,
    /// Still starting; the text says what it waits for.
    Waiting(String),
    /// Stopped for good: exited or dead without a restart. Waiting longer does not
    /// help.
    Stopped(String),
}

pub fn readiness(containers: &[&Container]) -> Readiness {
    let done =
        |c: &&&Container| c.state == ContainerState::Exited && c.status.starts_with("Exited (0)");
    let active: Vec<&&Container> = containers.iter().filter(|c| !done(c)).collect();
    if let Some(stopped) = active
        .iter()
        .find(|c| matches!(c.state, ContainerState::Exited | ContainerState::Dead))
    {
        return Readiness::Stopped(format!(
            "{} is {}: {}. Read its logs.",
            stopped.display_name(),
            stopped.state.label(),
            stopped.status
        ));
    }
    let waiting: Vec<String> = active
        .iter()
        .filter(|c| {
            c.state != ContainerState::Running
                || matches!(c.health, Some(Health::Starting | Health::Unhealthy))
        })
        .map(|c| format!("{} ({})", c.display_name(), status(c)))
        .collect();
    match waiting.as_slice() {
        [] => Readiness::Ready,
        _ => Readiness::Waiting(format!("Waiting for {}.", waiting.join(", "))),
    }
}

/// `running` or `running, starting`.
pub fn status(container: &Container) -> String {
    match container.health {
        Some(health) => format!("{}, {}", container.state.label(), health.label()),
        None => container.state.label().into(),
    }
}

impl WaitReport {
    pub fn text(&self) -> String {
        let head = if self.ready {
            format!(
                "{} is running and healthy after {} s.",
                self.target, self.waited_seconds
            )
        } else {
            format!(
                "{} is not ready after {} s.",
                self.target, self.waited_seconds
            )
        };
        let mut lines = vec![head];
        lines.extend(self.reason.clone());
        lines.extend(self.containers.iter().cloned());
        let lines: Vec<String> = lines.iter().map(|line| plain(line)).collect();
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests;

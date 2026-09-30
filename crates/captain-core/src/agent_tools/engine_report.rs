//! The `engine_status` answer: which engine Captain uses, whether it runs, and
//! what it has.

use schemars::JsonSchema;
use serde::Serialize;

use crate::format::bytes_label;
use crate::model::{Container, EngineInfo};
use crate::{HostResources, HostStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct EngineReport {
    /// `Captain Engine`, or `Other engine` for one Captain connects to but does not
    /// run.
    pub engine: String,
    /// running, stopped, starting, stopping, not-created, not-installed, failed, or
    /// unreachable.
    pub state: String,
    /// Why it is not installed, failed, or cannot be reached.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// The Docker endpoint, for example `unix:///…/docker.sock`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// The Docker Engine version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// For example `linux/aarch64`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    /// The size of Captain Engine's disk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk: Option<String>,
    /// True when Captain Engine runs Kubernetes (k3s).
    pub kubernetes: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub containers_running: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub containers_total: Option<usize>,
}

/// What the engine answered: its facts and containers, or why it did not answer.
pub type EngineAnswer = Result<(EngineInfo, Vec<Container>), String>;

/// The report. `host` is Captain Engine's state and resources, `None` for another
/// engine.
pub fn engine_report(
    engine: &str,
    host: Option<(&HostStatus, HostResources)>,
    answer: &EngineAnswer,
    kubernetes: bool,
) -> EngineReport {
    let (state, detail) = match (host, answer) {
        (Some((status, _)), _) if !status.is_running() => (
            status.key().to_string(),
            status.detail().map(ToString::to_string),
        ),
        (_, Ok(_)) => ("running".to_string(), None),
        (_, Err(why)) => ("unreachable".to_string(), Some(why.clone())),
    };
    let info = answer.as_ref().ok().map(|(info, _)| info);
    // Kubernetes pod sandboxes are left out, as in the lists.
    let containers: Option<Vec<&Container>> = answer
        .as_ref()
        .ok()
        .map(|(_, all)| all.iter().filter(|c| !c.is_sandbox()).collect());
    EngineReport {
        engine: engine.into(),
        state,
        detail,
        endpoint: info.map(|i| i.endpoint.clone()).filter(|e| !e.is_empty()),
        version: info.map(|i| i.version.clone()),
        platform: info.map(|i| format!("{}/{}", i.os, i.arch)),
        cpus: info.map(|i| i.cpus),
        memory: info.map(|i| bytes_label(i.memory_bytes)),
        disk: host.map(|(_, resources)| bytes_label(resources.disk_bytes)),
        kubernetes,
        containers_running: containers
            .as_ref()
            .map(|shown| shown.iter().filter(|c| c.state.is_active()).count()),
        containers_total: containers.as_ref().map(Vec::len),
    }
}

impl EngineReport {
    pub fn text(&self) -> String {
        let mut text = format!("{}: {}", self.engine, self.state);
        if let Some(detail) = &self.detail {
            text.push_str(&format!(" ({detail})"));
        }
        text.push('.');
        if let Some(version) = &self.version {
            text.push_str(&format!(" Docker {version}"));
            if let (Some(cpus), Some(memory)) = (self.cpus, &self.memory) {
                text.push_str(&format!(", {cpus} CPUs, {memory} memory"));
            }
            if let Some(disk) = &self.disk {
                text.push_str(&format!(", {disk} disk"));
            }
            text.push('.');
        }
        if let (Some(running), Some(total)) = (self.containers_running, self.containers_total) {
            text.push_str(&format!(" {running} of {total} containers running."));
        }
        text.push_str(if self.kubernetes {
            " Kubernetes is on."
        } else {
            " Kubernetes is off."
        });
        text
    }
}

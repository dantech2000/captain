//! The `list_containers` answer: one short row per container, never raw `inspect`.

use schemars::JsonSchema;
use serde::Serialize;

use super::untrusted::plain;
use crate::model::{Container, PortLink};
use crate::problems::needs_attention;

/// Every container but Kubernetes pod sandboxes, by project, then by name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ContainerList {
    pub containers: Vec<ContainerRow>,
}

/// One container. With `status_only`, only `name`, `state`, `health`, and
/// `needs_attention` are set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ContainerRow {
    /// The name, or `pod/container` for a Kubernetes pod container.
    pub name: String,
    /// created, running, paused, restarting, removing, exited, dead, or unknown.
    pub state: String,
    /// starting, healthy, or unhealthy; absent without a health check.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<String>,
    /// True while it restarts, crashed in the last minute, or fails its health check.
    #[serde(skip_serializing_if = "is_false")]
    pub needs_attention: bool,
    /// The first 12 characters of the ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The Compose project, or the Kubernetes namespace as `k8s:<namespace>`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// The Compose service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Published ports: a URL for web ports, else `localhost:PORT (Service)`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<String>,
    /// How long it runs, for example `3 hours`, or `Exited`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime: Option<String>,
}

fn is_false(value: &bool) -> bool {
    !value
}

/// The rows of `containers`. `crashed` says if a container crashed lately.
pub fn container_list(
    containers: &[Container],
    status_only: bool,
    crashed: &dyn Fn(&str) -> bool,
) -> ContainerList {
    let mut shown: Vec<&Container> = containers.iter().filter(|c| !c.is_sandbox()).collect();
    shown.sort_by_key(|c| (group_of(c), c.display_name()));
    let containers = shown
        .into_iter()
        .map(|c| {
            let status = ContainerRow {
                name: c.display_name(),
                state: c.state.label().into(),
                health: c.health.map(|h| h.label().into()),
                needs_attention: needs_attention(c, crashed(&c.id)),
                id: None,
                project: None,
                service: None,
                image: None,
                ports: Vec::new(),
                uptime: None,
            };
            if status_only {
                return status;
            }
            ContainerRow {
                id: Some(c.short_id().into()),
                project: group_of(c),
                service: c.compose.service.clone(),
                image: Some(c.image.clone()),
                ports: port_links(c),
                uptime: Some(c.uptime_label()),
                ..status
            }
        })
        .collect();
    ContainerList { containers }
}

/// The project a container belongs to, or its Kubernetes namespace.
pub(super) fn group_of(container: &Container) -> Option<String> {
    container.compose_project.clone().or_else(|| {
        container
            .kube_namespace
            .as_ref()
            .map(|namespace| format!("k8s:{namespace}"))
    })
}

/// The published ports as links, without duplicates.
pub(super) fn port_links(container: &Container) -> Vec<String> {
    let mut links: Vec<String> = Vec::new();
    for port in &container.ports {
        let Some(public) = port.public_port else {
            continue;
        };
        let link = match PortLink::of(public, port.private_port) {
            PortLink::Open(url) => url,
            PortLink::Copy { address, service } => format!("{address} ({service})"),
        };
        if !links.contains(&link) {
            links.push(link);
        }
    }
    links
}

impl ContainerList {
    /// One line per container, for clients that show only text.
    pub fn text(&self) -> String {
        let running = self
            .containers
            .iter()
            .filter(|c| c.state == "running")
            .count();
        let mut text = format!("{} containers, {running} running.", self.containers.len());
        for row in &self.containers {
            let mut parts = vec![row.name.clone(), row.state.clone()];
            parts.extend(row.health.clone());
            if row.needs_attention {
                parts.push("needs attention".into());
            }
            parts.extend(row.project.clone());
            parts.extend(row.image.clone());
            parts.extend(row.ports.iter().cloned());
            text.push('\n');
            text.push_str(&plain(&parts.join(" · ")));
        }
        text
    }
}

#[cfg(test)]
mod tests;

//! The containers, Captain Engine, and Kubernetes, with only what the menu shows.

use captain_core::HostStatus;
use captain_core::kubernetes::KubernetesStatus;
use captain_core::model::{Container, ContainerState, Health, PortLink};

use super::dot::Light;

/// One container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerEntry {
    pub id: String,
    pub name: String,
    /// The Compose service, else the name, for the Open Ports submenu.
    pub service: String,
    pub state: ContainerState,
    /// It crashed within the last minute, or its health check fails.
    pub failing: bool,
    pub project: Option<String>,
    /// One link per published host port.
    pub ports: Vec<PortLink>,
}

impl ContainerEntry {
    /// `crashed` comes from the event stream's crash tracker.
    pub fn of(container: &Container, crashed: bool) -> Self {
        let unhealthy = container.state == ContainerState::Running
            && container.health == Some(Health::Unhealthy);
        let mut ports = Vec::new();
        for port in &container.ports {
            let Some(public) = port.public_port else {
                continue;
            };
            let link = PortLink::of(public, port.private_port);
            if !ports.contains(&link) {
                ports.push(link);
            }
        }
        Self {
            id: container.id.clone(),
            name: container.name.clone(),
            service: container
                .compose
                .service
                .clone()
                .unwrap_or_else(|| container.display_name()),
            state: container.state,
            failing: crashed || unhealthy,
            project: container.compose_project.clone(),
            ports,
        }
    }

    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    pub fn light(&self) -> Light {
        if self.failing {
            return Light::Red;
        }
        match self.state {
            ContainerState::Running => Light::Green,
            ContainerState::Paused | ContainerState::Restarting => Light::Amber,
            _ => Light::Gray,
        }
    }
}

/// Captain Engine's state, when the settings choose it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostEntry {
    pub status: HostStatus,
    pub can_control: bool,
}

/// Kubernetes in Captain Engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubeEntry {
    /// The settings turn it on.
    pub enabled: bool,
    pub status: KubernetesStatus,
}

impl KubeEntry {
    /// "Kubernetes: Running · v1.31.4+k3s1".
    pub fn line(&self) -> String {
        match &self.status {
            KubernetesStatus::Running { version } => {
                format!("Kubernetes: Running \u{b7} {version}")
            }
            status => format!("Kubernetes: {}", status.label()),
        }
    }

    pub fn light(&self) -> Light {
        match self.status {
            KubernetesStatus::Running { .. } => Light::Green,
            KubernetesStatus::Starting => Light::Amber,
            KubernetesStatus::Failed(_) => Light::Red,
            KubernetesStatus::Off => Light::Gray,
        }
    }
}

use bollard::models::{ContainerSummary, PortSummary};
use std::collections::HashMap;

use captain_core::extension::{EXTENSION_LABEL, backend_extension};
use captain_core::model::{ComposeLabels, Container, ContainerState, Health, PortMapping};

const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";
const COMPOSE_SERVICE_LABEL: &str = "com.docker.compose.service";
const COMPOSE_WORKING_DIR_LABEL: &str = "com.docker.compose.project.working_dir";
const COMPOSE_CONFIG_FILES_LABEL: &str = "com.docker.compose.project.config_files";
/// Set by cri-dockerd on the containers of a Kubernetes pod.
const KUBE_NAMESPACE_LABEL: &str = "io.kubernetes.pod.namespace";

pub fn container(summary: ContainerSummary) -> Container {
    let name = summary
        .names
        .as_ref()
        .and_then(|names| names.first())
        .map(|name| name.trim_start_matches('/').to_string())
        .unwrap_or_default();
    let state = summary
        .state
        .map(|state| ContainerState::parse(state.as_ref()))
        .unwrap_or(ContainerState::Unknown);
    let mut ports: Vec<PortMapping> = summary
        .ports
        .unwrap_or_default()
        .into_iter()
        .map(port)
        .collect();
    ports.sort_by_key(|p| (p.private_port, p.public_port));

    let health = summary
        .health
        .and_then(|h| h.status)
        .and_then(|status| Health::parse(status.as_ref()));

    let mut labels = summary.labels.unwrap_or_default();
    let compose_project = labels.remove(COMPOSE_PROJECT_LABEL);
    let extension = backend_extension(
        labels.get(EXTENSION_LABEL).map(String::as_str),
        compose_project.as_deref(),
    );
    Container {
        id: summary.id.unwrap_or_default(),
        name,
        image: summary.image.unwrap_or_default(),
        state,
        status: summary.status.unwrap_or_default(),
        ports,
        created: summary.created.unwrap_or_default(),
        compose_project,
        compose: compose_labels(&mut labels),
        health,
        kube_namespace: labels.remove(KUBE_NAMESPACE_LABEL),
        extension,
    }
}

/// True if the container is running, paused, or restarting: it can write now, or
/// soon without anyone starting it.
pub fn is_active(summary: &ContainerSummary) -> bool {
    let state = summary.state.map(|s| ContainerState::parse(s.as_ref()));
    state.is_some_and(ContainerState::is_active)
}

fn compose_labels(labels: &mut HashMap<String, String>) -> ComposeLabels {
    let captain_override = labels.contains_key(ComposeLabels::CAPTAIN_OVERRIDE_LABEL);
    ComposeLabels {
        service: labels.remove(COMPOSE_SERVICE_LABEL),
        working_dir: labels.remove(COMPOSE_WORKING_DIR_LABEL),
        config_files: labels
            .remove(COMPOSE_CONFIG_FILES_LABEL)
            .map(|files| ComposeLabels::split_config_files(&files, captain_override))
            .unwrap_or_default(),
    }
}

fn port(port: PortSummary) -> PortMapping {
    let protocol = port.typ.map(|typ| typ.to_string()).unwrap_or_default();
    PortMapping {
        private_port: port.private_port,
        public_port: port.public_port,
        host_ip: port.ip.filter(|ip| !ip.is_empty()),
        protocol: if protocol.is_empty() {
            "tcp".into()
        } else {
            protocol
        },
    }
}

#[cfg(test)]
mod tests;

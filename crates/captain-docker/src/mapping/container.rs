use bollard::models::{ContainerSummary, PortSummary};
use captain_core::model::{Container, ContainerState, Health, PortMapping};

const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";

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

    Container {
        id: summary.id.unwrap_or_default(),
        name,
        image: summary.image.unwrap_or_default(),
        state,
        status: summary.status.unwrap_or_default(),
        ports,
        created: summary.created.unwrap_or_default(),
        compose_project: summary
            .labels
            .and_then(|mut labels| labels.remove(COMPOSE_PROJECT_LABEL)),
        health,
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

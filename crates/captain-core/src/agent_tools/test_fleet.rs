//! Containers like a real development machine's, for the size tests.

use crate::model::{ComposeLabels, Container, ContainerState, Health, PortMapping};

/// `count` containers in projects of four: web (port 8080), api, a Postgres
/// database (port 5432), and a worker.
pub fn fleet(count: usize) -> Vec<Container> {
    (0..count)
        .map(|i| {
            let project = format!("project{}", i / 4);
            let (service, image, port) = match i % 4 {
                0 => ("web", "nginx:1.27-alpine", Some((8080 + i as u16, 80))),
                1 => ("api", "ghcr.io/acme/api:2.14.0", None),
                2 => ("db", "postgres:17", Some((5432 + i as u16, 5432))),
                _ => ("worker", "ghcr.io/acme/worker:2.14.0", None),
            };
            Container {
                id: format!("{i:0>2}{}", "3f9a2b7c4d1e".repeat(5)),
                name: format!("{project}-{service}-1"),
                image: image.into(),
                state: ContainerState::Running,
                status: "Up 3 hours (healthy)".into(),
                ports: port
                    .map(|(public, private)| PortMapping {
                        private_port: private,
                        public_port: Some(public),
                        host_ip: Some("0.0.0.0".into()),
                        protocol: "tcp".into(),
                    })
                    .into_iter()
                    .collect(),
                created: 1_700_000_000,
                compose_project: Some(project),
                compose: ComposeLabels {
                    service: Some(service.into()),
                    working_dir: Some(format!("/Users/dev/code/project{}", i / 4)),
                    config_files: vec!["compose.yaml".into()],
                },
                health: Some(Health::Healthy),
                kube_namespace: None,
            }
        })
        .collect()
}

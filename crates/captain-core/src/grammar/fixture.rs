//! A sample catalog for the grammar tests.

use crate::kubernetes::{KubeService, ServicePort};
use crate::model::{ComposeLabels, Container, ContainerState, PortMapping};

use super::Catalog;

fn container(
    name: &str,
    project: Option<&str>,
    service: Option<&str>,
    ports: &[(u16, u16)],
) -> Container {
    Container {
        id: format!("{name}-id"),
        name: name.into(),
        image: "app:latest".into(),
        state: ContainerState::Running,
        status: "Up 3 hours".into(),
        ports: ports
            .iter()
            .map(|&(public, private)| PortMapping {
                private_port: private,
                public_port: Some(public),
                host_ip: None,
                protocol: "tcp".into(),
            })
            .collect(),
        created: 0,
        compose_project: project.map(Into::into),
        compose: ComposeLabels {
            service: service.map(Into::into),
            ..Default::default()
        },
        health: None,
        kube_namespace: None,
        extension: None,
    }
}

fn kube(namespace: &str, name: &str, ports: &[u16]) -> KubeService {
    KubeService {
        namespace: namespace.into(),
        name: name.into(),
        ports: ports
            .iter()
            .map(|&port| ServicePort {
                name: None,
                port,
                protocol: "TCP".into(),
            })
            .collect(),
    }
}

/// Project `shop` with web, api, two workers, and postgres; project `blog` with its
/// own api; a loose `redis`; a stopped loose container named `blog`; a pod
/// container; and three Kubernetes services, two of them named `web`.
pub fn catalog() -> Catalog {
    let shop =
        |name, service, ports: &[(u16, u16)]| container(name, Some("shop"), Some(service), ports);
    let mut stopped = container("blog", None, None, &[]);
    stopped.state = ContainerState::Exited;
    let mut pod = container("k8s_web_web-1", None, None, &[]);
    pod.kube_namespace = Some("default".into());
    Catalog {
        containers: vec![
            shop("shop-web-1", "web", &[(8080, 80)]),
            shop("shop-api-1", "api", &[(3000, 3000)]),
            shop("shop-worker-1", "worker", &[]),
            shop("shop-worker-2", "worker", &[]),
            shop("shop-postgres-1", "postgres", &[(5432, 5432)]),
            container("blog-api-1", Some("blog"), Some("api"), &[]),
            container("redis", None, None, &[]),
            stopped,
            pod,
        ],
        kube_services: vec![
            kube("default", "web", &[80]),
            kube("default", "api", &[80, 443]),
            kube("other", "web", &[80]),
        ],
        current_project: None,
        kubernetes: true,
        compose: true,
    }
}

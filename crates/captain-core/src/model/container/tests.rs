use super::{Container, ContainerState};
use crate::model::PortMapping;

#[test]
fn states_parse_and_their_labels_round_trip() {
    assert_eq!(ContainerState::parse("running"), ContainerState::Running);
    assert_eq!(ContainerState::parse("exited"), ContainerState::Exited);
    assert_eq!(ContainerState::parse("bogus"), ContainerState::Unknown);
    for state in [
        ContainerState::Created,
        ContainerState::Running,
        ContainerState::Dead,
    ] {
        assert_eq!(ContainerState::parse(state.label()), state);
    }
}

#[test]
fn short_id_and_ports_label() {
    let tcp = |public| PortMapping {
        private_port: 80,
        public_port: Some(public),
        host_ip: Some("0.0.0.0".into()),
        protocol: "tcp".into(),
    };
    let container = Container {
        id: "0123456789abcdef0123".into(),
        name: "web".into(),
        image: "nginx".into(),
        state: ContainerState::Running,
        status: "Up".into(),
        ports: vec![tcp(8080), tcp(8080), tcp(8081)],
        created: 0,
        compose_project: None,
        compose: Default::default(),
        health: None,
        kube_namespace: None,
        extension: None,
    };

    assert_eq!(container.short_id(), "0123456789ab");
    assert_eq!(
        container.ports_label(),
        "0.0.0.0:8080->80/tcp, 0.0.0.0:8081->80/tcp"
    );
}

#[test]
fn published_ports_skip_unpublished_and_duplicates() {
    let port = |private_port, public_port| PortMapping {
        private_port,
        public_port,
        host_ip: None,
        protocol: "tcp".into(),
    };
    let container = Container {
        id: "abc".into(),
        name: "web".into(),
        image: "nginx".into(),
        state: ContainerState::Running,
        status: "Up".into(),
        ports: vec![port(80, Some(8080)), port(80, Some(8080)), port(443, None)],
        created: 0,
        compose_project: None,
        compose: Default::default(),
        health: None,
        kube_namespace: None,
        extension: None,
    };

    assert_eq!(container.published_ports(), [8080]);
}

fn with_image(image: &str) -> Container {
    Container {
        id: "abc".into(),
        name: "c".into(),
        image: image.into(),
        state: ContainerState::Running,
        status: "Up 3 hours (healthy)".into(),
        ports: Vec::new(),
        created: 0,
        compose_project: None,
        compose: Default::default(),
        health: None,
        kube_namespace: None,
        extension: None,
    }
}

#[test]
fn image_name_and_tag_handle_registries_and_digests() {
    assert_eq!(
        with_image("nginx:1.27-alpine").image_name_and_tag(),
        ("nginx", "1.27-alpine")
    );
    assert_eq!(
        with_image("minio/minio").image_name_and_tag(),
        ("minio/minio", "")
    );
    assert_eq!(
        with_image("localhost:5000/app:dev").image_name_and_tag(),
        ("localhost:5000/app", "dev")
    );
    assert_eq!(
        with_image("sha256:abc").image_name_and_tag(),
        ("sha256:abc", "")
    );
}

#[test]
fn uptime_label_strips_the_prefix_and_health() {
    assert_eq!(with_image("x").uptime_label(), "3 hours");
    let exited = Container {
        state: ContainerState::Exited,
        ..with_image("x")
    };
    assert_eq!(exited.uptime_label(), "Exited");
}

use super::{Container, ContainerState};
use crate::model::PortMapping;

#[test]
fn parse_known_and_unknown_states() {
    assert_eq!(ContainerState::parse("running"), ContainerState::Running);
    assert_eq!(ContainerState::parse("exited"), ContainerState::Exited);
    assert_eq!(ContainerState::parse("bogus"), ContainerState::Unknown);
}

#[test]
fn state_label_round_trips() {
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
    };

    assert_eq!(container.short_id(), "0123456789ab");
    assert_eq!(
        container.ports_label(),
        "0.0.0.0:8080->80/tcp, 0.0.0.0:8081->80/tcp"
    );
}

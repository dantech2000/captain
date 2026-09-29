use captain_core::EngineError;
use captain_core::model::{Container, ContainerState, PortMapping};
use captain_ui::Connection;

use super::{EngineStatus, TraySnapshot};

fn container(name: &str, state: ContainerState, project: Option<&str>) -> Container {
    Container {
        id: format!("{name}-id"),
        name: name.into(),
        image: "nginx:latest".into(),
        state,
        status: String::new(),
        ports: Vec::new(),
        created: 0,
        compose_project: project.map(Into::into),
        health: None,
        compose: Default::default(),
    }
}

#[test]
fn engine_status_follows_the_connection() {
    assert_eq!(
        EngineStatus::of(&Connection::Connecting),
        EngineStatus::Starting
    );
    let failed = Connection::Failed(EngineError::Unreachable("gone".into()));
    assert_eq!(EngineStatus::of(&failed), EngineStatus::Stopped);
    assert_eq!(EngineStatus::Running.label(), "Captain Engine is running");
}

#[test]
fn a_stopped_engine_has_no_containers() {
    let containers = [container("web", ContainerState::Running, None)];
    let snapshot = TraySnapshot::new(EngineStatus::Stopped, &containers);
    assert!(snapshot.containers.is_empty());
    assert_eq!(snapshot.active_count(), 0);
}

#[test]
fn a_running_engine_keeps_ids_states_projects_and_ports() {
    let mut web = container("web", ContainerState::Running, Some("shop"));
    web.ports = vec![
        PortMapping {
            private_port: 80,
            public_port: Some(8080),
            host_ip: Some("0.0.0.0".into()),
            protocol: "tcp".into(),
        },
        PortMapping {
            private_port: 80,
            public_port: Some(8080),
            host_ip: Some("::".into()),
            protocol: "tcp".into(),
        },
    ];
    let db = container("db", ContainerState::Exited, None);
    let snapshot = TraySnapshot::new(EngineStatus::Running, &[web, db]);

    assert_eq!(snapshot.containers.len(), 2);
    assert_eq!(snapshot.containers[0].id, "web-id");
    assert_eq!(snapshot.containers[0].project.as_deref(), Some("shop"));
    assert_eq!(snapshot.containers[0].ports, [8080]);
    assert_eq!(snapshot.active_count(), 1);
}

#[test]
fn snapshots_compare_equal_when_nothing_the_menu_shows_changed() {
    let mut a = container("web", ContainerState::Running, None);
    let mut b = a.clone();
    a.status = "Up 3 minutes".into();
    b.status = "Up 4 minutes".into();
    let first = TraySnapshot::new(EngineStatus::Running, &[a]);
    let second = TraySnapshot::new(EngineStatus::Running, &[b.clone()]);
    assert_eq!(first, second);

    b.state = ContainerState::Exited;
    let third = TraySnapshot::new(EngineStatus::Running, &[b]);
    assert_ne!(first, third);
}

use captain_core::EngineError;
use captain_core::model::{Container, ContainerState, Health, PortLink, PortMapping};
use captain_core::problems::Problem;
use captain_ui::Connection;

use captain_core::HostStatus;
use captain_core::model::EngineInfo;

use super::{EngineStatus, TraySnapshot, status_line};

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
        kube_namespace: None,
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
    assert_eq!(
        snapshot.containers[0].ports,
        [PortLink::Open("http://localhost:8080".into())]
    );
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

    // A new status line alone keeps the menu; its text changes in place.
    let mut busier = first.clone();
    busier.status_line = "Captain Engine: Running \u{b7} 2 CPUs \u{b7} 90 MB of 2.0 GB".into();
    assert!(first.same_menu(&busier));

    b.state = ContainerState::Exited;
    let third = TraySnapshot::new(EngineStatus::Running, &[b]);
    assert_ne!(first, third);
    assert!(!first.same_menu(&third));
}

#[test]
fn the_status_line_names_the_engine_and_its_use() {
    let info = EngineInfo {
        cpus: 5,
        memory_bytes: 5_900 << 20,
        ..EngineInfo::default()
    };
    let connected = Connection::Connected(info);
    assert_eq!(
        status_line(&connected, 182 << 20, Some(&HostStatus::Running)),
        "Captain Engine: Running \u{b7} 5 CPUs \u{b7} 182 MB of 5.8 GB"
    );
    let failed = HostStatus::Failed("no VM".into());
    assert_eq!(
        status_line(&Connection::Connecting, 0, Some(&failed)),
        "Captain Engine: Did not start"
    );
}

#[test]
fn captain_engine_status_comes_from_the_host() {
    let failed = Connection::Failed(EngineError::Unreachable("gone".into()));
    assert_eq!(
        EngineStatus::of_host(&HostStatus::Starting, &failed),
        EngineStatus::Starting
    );
    assert_eq!(
        EngineStatus::of_host(&HostStatus::Running, &Connection::Connecting),
        EngineStatus::Starting
    );
    assert_eq!(
        EngineStatus::of_host(&HostStatus::Running, &failed),
        EngineStatus::Stopped
    );
    assert_eq!(
        EngineStatus::of_host(&HostStatus::NotCreated, &Connection::Connecting),
        EngineStatus::Stopped
    );
    let connected = Connection::Connected(EngineInfo::default());
    assert_eq!(
        EngineStatus::of_host(&HostStatus::Running, &connected),
        EngineStatus::Running
    );
}

#[test]
fn the_icon_needs_attention_for_a_failed_host_or_a_sick_container() {
    let failed = HostStatus::Failed("no VM".into());
    let status = EngineStatus::of_host(&failed, &Connection::Connecting);
    assert_eq!(status, EngineStatus::NeedsAttention);

    let mut web = container("web", ContainerState::Running, None);
    let healthy = TraySnapshot::new(EngineStatus::Running, std::slice::from_ref(&web));
    assert_eq!(healthy.icon(), EngineStatus::Running);
    web.health = Some(Health::Unhealthy);
    let sick = TraySnapshot::new(EngineStatus::Running, &[web]);
    assert_eq!(sick.engine, EngineStatus::Running);
    assert_eq!(sick.icon(), EngineStatus::NeedsAttention);
    assert_eq!(
        sick.problem.as_ref().map(Problem::line).as_deref(),
        Some("web fails its health check. The logs say why; a restart often helps.")
    );
}

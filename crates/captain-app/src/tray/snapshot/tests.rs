use captain_core::model::{Container, ContainerState, PortLink, PortMapping};
use captain_ui::{Connection, EngineHealth};

use captain_core::HostStatus;
use captain_core::model::EngineInfo;

use super::{TraySnapshot, status_line};

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
        extension: None,
        compose: Default::default(),
    }
}

#[test]
fn a_running_engine_keeps_ids_states_projects_and_ports_and_a_stopped_one_none() {
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
    let snapshot = TraySnapshot::new(EngineHealth::Running, &[web, db]);

    assert_eq!(snapshot.containers.len(), 2);
    assert_eq!(snapshot.containers[0].id, "web-id");
    assert_eq!(snapshot.containers[0].project.as_deref(), Some("shop"));
    assert_eq!(
        snapshot.containers[0].ports,
        [PortLink::Open("http://localhost:8080".into())]
    );
    assert_eq!(snapshot.active_count(), 1);

    let web = container("web", ContainerState::Running, None);
    let snapshot = TraySnapshot::new(EngineHealth::Stopped, &[web]);
    assert!(snapshot.containers.is_empty());
    assert_eq!(snapshot.active_count(), 0);
}

#[test]
fn snapshots_compare_equal_when_nothing_the_menu_shows_changed() {
    let mut a = container("web", ContainerState::Running, None);
    let mut b = a.clone();
    a.status = "Up 3 minutes".into();
    b.status = "Up 4 minutes".into();
    let first = TraySnapshot::new(EngineHealth::Running, &[a]);
    let second = TraySnapshot::new(EngineHealth::Running, &[b.clone()]);
    assert_eq!(first, second);

    // A new status line alone keeps the menu; its text changes in place.
    let mut busier = first.clone();
    busier.status_line = "Captain Engine: Running \u{b7} 2 CPUs \u{b7} 90 MB of 2.0 GB".into();
    assert!(first.same_menu(&busier));

    b.state = ContainerState::Exited;
    let third = TraySnapshot::new(EngineHealth::Running, &[b]);
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
        status_line(
            EngineHealth::Running,
            &connected,
            182 << 20,
            Some(&HostStatus::Running)
        ),
        "Captain Engine: Running \u{b7} 5 CPUs \u{b7} 182 MB of 5.8 GB"
    );
    let failed = HostStatus::Failed("no VM".into());
    assert_eq!(
        status_line(
            EngineHealth::CannotRun,
            &Connection::Connecting,
            0,
            Some(&failed)
        ),
        "Captain Engine: Did not start"
    );
    let line = |engine| {
        status_line(
            engine,
            &Connection::Connecting,
            0,
            Some(&HostStatus::Running),
        )
    };
    assert_eq!(
        line(EngineHealth::Reconnecting),
        "Captain Engine: Reconnecting\u{2026}"
    );
    assert_eq!(
        line(EngineHealth::NotAnswering),
        "Captain Engine: Not answering"
    );
}

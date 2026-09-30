use captain_core::model::{ContainerAction, ContainerState};

use super::{TrayCommand, TrayItem, build};
use captain_core::HostStatus;

use crate::tray::snapshot::{ContainerEntry, EngineStatus, HostEntry, TraySnapshot};

fn entry(name: &str, state: ContainerState, project: Option<&str>) -> ContainerEntry {
    ContainerEntry {
        id: format!("{name}-id"),
        name: name.into(),
        state,
        project: project.map(Into::into),
        ports: Vec::new(),
        health: None,
    }
}

fn snapshot(containers: Vec<ContainerEntry>) -> TraySnapshot {
    TraySnapshot {
        engine: EngineStatus::Running,
        containers,
        host: None,
        contexts: Default::default(),
    }
}

fn labels(items: &[TrayItem]) -> Vec<String> {
    items
        .iter()
        .map(|item| match item {
            TrayItem::Label(label)
            | TrayItem::Command { label, .. }
            | TrayItem::Check { label, .. }
            | TrayItem::Submenu { label, .. } => label.clone(),
            TrayItem::Separator => "-".into(),
        })
        .collect()
}

fn submenu<'a>(items: &'a [TrayItem], name: &str) -> &'a [TrayItem] {
    items
        .iter()
        .find_map(|item| match item {
            TrayItem::Submenu { label, items } if label == name => Some(items.as_slice()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no submenu {name}"))
}

#[test]
fn a_stopped_engine_shows_status_open_settings_and_quit() {
    let menu = build(&TraySnapshot {
        engine: EngineStatus::Stopped,
        containers: Vec::new(),
        host: None,
        contexts: Default::default(),
    });
    assert_eq!(
        labels(&menu),
        [
            "Captain Engine is stopped",
            "-",
            "Open Captain",
            "Settings\u{2026}",
            "-",
            "Quit Captain",
        ]
    );
}

#[test]
fn a_running_engine_counts_containers_and_adds_the_submenus() {
    let menu = build(&snapshot(vec![
        entry("web", ContainerState::Running, None),
        entry("cache", ContainerState::Paused, None),
        entry("db", ContainerState::Exited, None),
    ]));
    assert_eq!(
        labels(&menu),
        [
            "Captain Engine is running",
            "2 of 3 containers running",
            "-",
            "Open Captain",
            "Settings\u{2026}",
            "-",
            "Containers",
            "Projects",
            "-",
            "Quit Captain",
        ]
    );
    assert_eq!(labels(submenu(&menu, "Containers")), ["web", "cache"]);
    assert_eq!(labels(submenu(&menu, "Projects")), ["No Compose projects"]);
}

#[test]
fn one_container_is_singular() {
    let menu = build(&snapshot(vec![entry("web", ContainerState::Exited, None)]));
    assert_eq!(labels(&menu)[1], "0 of 1 container running");
    assert_eq!(
        labels(submenu(&menu, "Containers")),
        ["No running containers"]
    );
}

#[test]
fn a_container_has_stop_restart_and_its_ports() {
    let mut web = entry("web", ContainerState::Running, None);
    web.ports = vec![8080, 8443];
    let menu = build(&snapshot(vec![web]));
    let web = submenu(submenu(&menu, "Containers"), "web");
    assert_eq!(
        labels(web),
        [
            "Stop",
            "Restart",
            "-",
            "Open localhost:8080 in browser",
            "Open localhost:8443 in browser",
        ]
    );
    assert!(matches!(
        &web[0],
        TrayItem::Command {
            command: TrayCommand::Container { id, action: ContainerAction::Stop },
            ..
        } if id == "web-id"
    ));
    assert!(matches!(
        &web[4],
        TrayItem::Command {
            command: TrayCommand::OpenPort(8443),
            ..
        }
    ));
}

#[test]
fn projects_act_on_the_containers_that_can_take_the_action() {
    let menu = build(&snapshot(vec![
        entry("shop-web", ContainerState::Running, Some("shop")),
        entry("shop-db", ContainerState::Exited, Some("shop")),
        entry("blog-web", ContainerState::Running, Some("blog")),
        entry("loose", ContainerState::Running, None),
    ]));
    let projects = submenu(&menu, "Projects");
    assert_eq!(labels(projects), ["blog (1/1)", "shop (1/2)"]);

    let shop = submenu(projects, "shop (1/2)");
    let commands: Vec<(Vec<String>, ContainerAction, bool)> = shop
        .iter()
        .map(|item| match item {
            TrayItem::Command {
                command: TrayCommand::Project { ids, action },
                enabled,
                ..
            } => (ids.clone(), *action, *enabled),
            other => panic!("unexpected item {other:?}"),
        })
        .collect();
    assert_eq!(
        commands,
        [
            (vec!["shop-db-id".into()], ContainerAction::Start, true),
            (vec!["shop-web-id".into()], ContainerAction::Stop, true),
            (
                vec!["shop-web-id".into(), "shop-db-id".into()],
                ContainerAction::Restart,
                true
            ),
        ]
    );

    let blog = submenu(projects, "blog (1/1)");
    assert!(matches!(&blog[0], TrayItem::Command { enabled: false, .. }));
}

fn with_host(engine: EngineStatus, status: HostStatus) -> TraySnapshot {
    TraySnapshot {
        engine,
        containers: Vec::new(),
        host: Some(HostEntry {
            status,
            can_control: true,
        }),
        contexts: Default::default(),
    }
}

#[test]
fn a_stopped_captain_engine_offers_start() {
    let menu = build(&with_host(EngineStatus::Stopped, HostStatus::Stopped));
    assert_eq!(
        labels(&menu)[..2],
        ["Captain Engine is stopped", "Start Captain Engine"]
    );
    assert!(menu.contains(&TrayItem::Command {
        label: "Start Captain Engine".into(),
        command: TrayCommand::StartEngine,
        enabled: true,
    }));
}

#[test]
fn a_running_captain_engine_offers_stop() {
    let menu = build(&with_host(EngineStatus::Running, HostStatus::Running));
    assert_eq!(
        labels(&menu)[..3],
        [
            "Captain Engine is running",
            "0 of 0 containers running",
            "Stop Captain Engine"
        ]
    );
}

#[test]
fn a_new_captain_engine_opens_setup() {
    let menu = build(&with_host(EngineStatus::Stopped, HostStatus::NotCreated));
    assert_eq!(labels(&menu)[0], "Captain Engine is not set up");
    assert!(menu.contains(&TrayItem::Command {
        label: "Set Up Captain Engine\u{2026}".into(),
        command: TrayCommand::OpenCaptain,
        enabled: true,
    }));
}

#[test]
fn another_engine_has_no_engine_item() {
    let menu = build(&snapshot(Vec::new()));
    assert!(
        !labels(&menu)
            .iter()
            .any(|label| label.contains("Captain Engine\u{2026}"))
    );
    assert!(!menu.iter().any(|item| matches!(
        item,
        TrayItem::Command {
            command: TrayCommand::StartEngine | TrayCommand::StopEngine,
            ..
        }
    )));
}

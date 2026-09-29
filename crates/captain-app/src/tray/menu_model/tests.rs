use captain_core::model::{ContainerAction, ContainerState};

use super::{TrayCommand, TrayItem, build};
use crate::tray::snapshot::{ContainerEntry, EngineStatus, TraySnapshot};

fn entry(name: &str, state: ContainerState, project: Option<&str>) -> ContainerEntry {
    ContainerEntry {
        id: format!("{name}-id"),
        name: name.into(),
        state,
        project: project.map(Into::into),
        ports: Vec::new(),
    }
}

fn snapshot(containers: Vec<ContainerEntry>) -> TraySnapshot {
    TraySnapshot {
        engine: EngineStatus::Running,
        containers,
    }
}

fn labels(items: &[TrayItem]) -> Vec<String> {
    items
        .iter()
        .map(|item| match item {
            TrayItem::Label(label)
            | TrayItem::Command { label, .. }
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

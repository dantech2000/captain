use captain_core::HostStatus;
use captain_core::model::{ContainerAction, ContainerState};

use super::{TrayCommand, TrayItem, build};
use crate::tray::dot::Light;
use crate::tray::entries::HostEntry;
use crate::tray::snapshot::TraySnapshot;
use crate::tray::test_support::{entry, find, labels, light, snapshot, submenu};
use captain_ui::EngineHealth;

#[test]
fn a_stopped_engine_shows_status_open_settings_and_quit() {
    let menu = build(&TraySnapshot {
        engine: EngineHealth::Stopped,
        status_line: "Captain Engine: Stopped".into(),
        ..snapshot(Vec::new())
    });
    assert_eq!(
        labels(&menu),
        [
            "Captain Engine: Stopped",
            "-",
            "Open Captain",
            "Settings\u{2026}",
            "-",
            "Quit Captain",
        ]
    );
    assert_eq!(light(&menu[0]), Some(Light::Gray));
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
            "Captain Engine: Running",
            "2 of 3 containers running",
            "-",
            "Open Captain",
            "Settings\u{2026}",
            "-",
            "Containers",
            "Projects",
            "Open Ports",
            "-",
            "Stop All Containers",
            "-",
            "Quit Captain",
        ]
    );
    assert_eq!(labels(submenu(&menu, "Containers")), ["web", "cache"]);
    assert_eq!(labels(submenu(&menu, "Projects")), ["No Compose Projects"]);
    assert_eq!(labels(submenu(&menu, "Open Ports")), ["No Open Ports"]);
}

#[test]
fn stop_all_is_disabled_when_nothing_runs() {
    let menu = build(&snapshot(vec![entry("web", ContainerState::Exited, None)]));
    assert_eq!(labels(&menu)[1], "0 of 1 container running");
    assert!(matches!(
        find(&menu, "Stop All Containers"),
        TrayItem::Command { enabled: false, .. }
    ));
    let menu = build(&snapshot(vec![entry("web", ContainerState::Running, None)]));
    assert_eq!(
        find(&menu, "Stop All Containers"),
        &TrayItem::Command {
            label: "Stop All Containers".into(),
            command: TrayCommand::Containers {
                ids: vec!["web-id".into()],
                action: ContainerAction::Stop,
            },
            enabled: true,
        }
    );
}

#[test]
fn each_row_has_its_status_light() {
    let mut crashing = entry("api", ContainerState::Running, Some("shop"));
    crashing.failing = true;
    let menu = build(&snapshot(vec![
        entry("web", ContainerState::Running, Some("blog")),
        entry("cache", ContainerState::Paused, Some("cache")),
        crashing,
        entry("db", ContainerState::Exited, Some("shop")),
        entry("old", ContainerState::Exited, Some("old")),
    ]));
    assert_eq!(light(&menu[0]), Some(Light::Green));
    let containers = submenu(&menu, "Containers");
    let lights: Vec<_> = containers.iter().map(light).collect();
    assert_eq!(
        lights,
        [Some(Light::Green), Some(Light::Amber), Some(Light::Red)]
    );
    let projects = submenu(&menu, "Projects");
    let lights: Vec<_> = projects.iter().map(light).collect();
    // In name order: blog runs, cache is paused, old is stopped, shop fails.
    assert_eq!(
        lights,
        [
            Some(Light::Green),
            Some(Light::Green),
            Some(Light::Gray),
            Some(Light::Red)
        ]
    );
}

fn with_host(engine: EngineHealth, status: HostStatus) -> TraySnapshot {
    TraySnapshot {
        engine,
        host: Some(HostEntry {
            status,
            can_control: true,
        }),
        ..snapshot(Vec::new())
    }
}

#[test]
fn the_engine_item_follows_the_captain_engine_state() {
    let menu = build(&with_host(EngineHealth::Stopped, HostStatus::Stopped));
    assert_eq!(labels(&menu)[2], "Start Captain Engine");
    assert!(menu.contains(&TrayItem::command(
        "Start Captain Engine",
        TrayCommand::StartEngine
    )));
    let menu = build(&with_host(EngineHealth::Running, HostStatus::Running));
    assert_eq!(labels(&menu)[3], "Stop Captain Engine");
    let menu = build(&with_host(EngineHealth::Stopped, HostStatus::NotCreated));
    assert!(menu.contains(&TrayItem::command(
        "Set Up Captain Engine\u{2026}",
        TrayCommand::OpenCaptain
    )));
    // Another engine has no engine item.
    let menu = build(&snapshot(Vec::new()));
    assert!(!menu.iter().any(|item| matches!(
        item,
        TrayItem::Command {
            command: TrayCommand::StartEngine | TrayCommand::StopEngine,
            ..
        }
    )));
}

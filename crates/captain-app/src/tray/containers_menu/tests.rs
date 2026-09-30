use captain_core::model::{ContainerState, PortLink};

use crate::tray::menu_model::{TrayCommand, TrayItem, build};
use crate::tray::test_support::{entry, labels, snapshot, submenu};

#[test]
fn a_container_has_stop_restart_logs_and_its_ports() {
    let mut web = entry("web", ContainerState::Running, None);
    web.ports = vec![
        PortLink::Open("http://localhost:8080".into()),
        PortLink::of(5432, 5432),
    ];
    let menu = build(&snapshot(vec![web]));
    let web = submenu(submenu(&menu, "Containers"), "web");
    assert_eq!(
        labels(web),
        [
            "Stop",
            "Restart",
            "Show Logs in a Window",
            "-",
            "Open localhost:8080 in Browser",
            "Copy localhost:5432",
        ]
    );
    assert_eq!(
        web[2],
        TrayItem::command(
            "Show Logs in a Window",
            TrayCommand::FloatLog {
                id: "web-id".into(),
                name: "web".into(),
            }
        )
    );
}

#[test]
fn open_ports_open_web_pages_and_copy_database_addresses() {
    let mut web = entry("shop-web-1", ContainerState::Running, Some("shop"));
    web.service = "web".into();
    web.ports = vec![PortLink::of(8080, 80)];
    let mut db = entry("shop-db-1", ContainerState::Running, Some("shop"));
    db.service = "db".into();
    db.ports = vec![PortLink::of(5432, 5432)];
    let mut stopped = entry("old", ContainerState::Exited, None);
    stopped.ports = vec![PortLink::of(9000, 80)];
    let menu = build(&snapshot(vec![web, db, stopped]));
    assert_eq!(
        submenu(&menu, "Open Ports"),
        [
            TrayItem::command(
                "web \u{2014} localhost:8080",
                TrayCommand::OpenUrl("http://localhost:8080".into())
            ),
            TrayItem::command(
                "db \u{2014} Copy localhost:5432",
                TrayCommand::CopyAddress("localhost:5432".into())
            ),
        ]
    );
}

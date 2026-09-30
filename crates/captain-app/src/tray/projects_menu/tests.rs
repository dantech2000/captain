use captain_core::model::{ContainerAction, ContainerState};

use crate::tray::menu_model::{TrayCommand, TrayItem, build};
use crate::tray::test_support::{entry, labels, snapshot, submenu};

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
    let commands: Vec<(&str, Vec<String>, ContainerAction)> = shop
        .iter()
        .map(|item| match item {
            TrayItem::Command {
                label,
                command: TrayCommand::Containers { ids, action },
                ..
            } => (label.as_str(), ids.clone(), *action),
            other => panic!("unexpected item {other:?}"),
        })
        .collect();
    assert_eq!(
        commands,
        [
            ("Start", vec!["shop-db-id".into()], ContainerAction::Start),
            ("Stop", vec!["shop-web-id".into()], ContainerAction::Stop),
            (
                "Restart",
                vec!["shop-web-id".into(), "shop-db-id".into()],
                ContainerAction::Restart
            ),
        ]
    );

    let blog = submenu(projects, "blog (1/1)");
    assert!(matches!(&blog[0], TrayItem::Command { enabled: false, .. }));
}

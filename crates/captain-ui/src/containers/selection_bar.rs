use captain_core::model::{Container, ContainerAction, ContainerState};
use gpui_kit::*;

use super::bulk_delete_dialog;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, selection_bar, text_button};
use crate::workspace::Workspace;

/// Start, Stop, Restart, and Delete for the selected containers. Each action runs
/// on the containers it applies to, and its button is disabled when none apply.
pub fn render(handle: &Entity<Workspace>, workspace: &Workspace, palette: &Palette) -> Div {
    let containers = workspace.bulk_containers();
    let action = |action: ContainerAction, applies: fn(&Container) -> bool| {
        let targets: Vec<_> = containers
            .iter()
            .filter(|c| applies(c) && !workspace.is_pending(&c.id))
            .map(|c| (c.clone(), action))
            .collect();
        let handle = handle.clone();
        text_button(
            SharedString::from(format!("bulk-{}", action.label())),
            action.label(),
            ButtonTone::Accent,
            !targets.is_empty(),
            palette,
            move |_, _, cx| {
                let targets = targets.clone();
                handle.update(cx, |workspace, cx| workspace.run_bulk(targets, action, cx));
            },
        )
    };
    let delete = {
        let handle = handle.clone();
        let targets = containers.clone();
        text_button(
            "bulk-delete",
            ContainerAction::Remove.label(),
            ButtonTone::Danger,
            !containers.is_empty(),
            palette,
            move |_, window, cx| {
                bulk_delete_dialog::open(targets.clone(), handle.clone(), window, cx)
            },
        )
    };
    let clear = handle.clone();

    selection_bar(
        containers.len(),
        vec![
            action(ContainerAction::Start, |c| !c.state.is_active()),
            action(ContainerAction::Stop, |c| c.state.is_active()),
            action(ContainerAction::Restart, |c| {
                c.state == ContainerState::Running
            }),
            delete,
        ],
        palette,
        move |_, _, cx| clear.update(cx, |workspace, cx| workspace.clear_bulk(cx)),
    )
}

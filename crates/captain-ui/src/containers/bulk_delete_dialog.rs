use captain_core::model::{Container, ContainerAction, count_label};
use gpui_kit::component::WindowExt;
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::*;

use crate::workspace::Workspace;

/// Asks once before deleting several containers, and lists them. Live containers
/// are stopped and removed, like "Stop and delete" for one.
pub fn open(
    containers: Vec<Container>,
    handle: Entity<Workspace>,
    window: &mut Window,
    cx: &mut App,
) {
    let running = containers.iter().filter(|c| c.state.is_active()).count();
    let title = SharedString::from(format!(
        "Delete {}?",
        count_label(containers.len(), "container")
    ));
    let names = containers
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let note = match running {
        0 => "Their volumes stay.".to_string(),
        n => format!(
            "{} running. Captain stops them first. Their volumes stay.",
            if n == 1 {
                "1 is".to_string()
            } else {
                format!("{n} are")
            }
        ),
    };
    let description = SharedString::from(format!("{names}\n\n{note}"));
    let targets: Vec<_> = containers
        .into_iter()
        .map(|c| {
            let action = ContainerAction::removal_for(c.state);
            (c, action)
        })
        .collect();

    window.open_alert_dialog(cx, move |alert, _, _| {
        let handle = handle.clone();
        let targets = targets.clone();
        alert
            .title(title.clone())
            .description(description.clone())
            .show_cancel(true)
            .ok_text(ContainerAction::Remove.label())
            .ok_variant(ButtonVariant::Danger)
            .on_ok(move |_, _, cx| {
                let targets = targets.clone();
                handle.update(cx, |workspace, cx| {
                    workspace.run_bulk(targets, ContainerAction::Remove, cx)
                });
                true
            })
    });
}

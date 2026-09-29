use captain_core::model::{Container, ContainerAction};
use gpui_kit::component::WindowExt;
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::*;

use crate::workspace::Workspace;

/// Asks before deleting `container`. A live container can only be deleted here, with
/// "Stop and delete", which force-removes it.
pub fn open(container: &Container, handle: Entity<Workspace>, window: &mut Window, cx: &mut App) {
    let id = container.id.clone();
    let action = ContainerAction::removal_for(container.state);
    let title = SharedString::from(format!("Delete {}?", container.name));
    let description = SharedString::from(match action {
        ContainerAction::ForceRemove => format!(
            "{} is running. Captain stops it, then removes the container. Its volumes stay.",
            container.name
        ),
        _ => "This removes the container. Its volumes stay.".to_string(),
    });

    window.open_alert_dialog(cx, move |alert, _, _| {
        let handle = handle.clone();
        let id = id.clone();
        alert
            .title(title.clone())
            .description(description.clone())
            .show_cancel(true)
            .ok_text(action.label())
            .ok_variant(ButtonVariant::Danger)
            .on_ok(move |_, _, cx| {
                handle.update(cx, |workspace, cx| {
                    workspace.run_action(id.clone(), action, cx)
                });
                true
            })
    });
}

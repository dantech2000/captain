use captain_core::model::{Container, ContainerAction};
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use crate::widgets::danger_footer;
use crate::workspace::Workspace;

/// Asks before deleting `container`. A live container can only be deleted here, with
/// "Stop and delete", which force-removes it. Confirming does nothing once the
/// workspace switched engines.
pub fn open(container: &Container, handle: Entity<Workspace>, window: &mut Window, cx: &mut App) {
    let generation = handle.read(cx).engine_generation();
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
            .footer(danger_footer(action.label(), description.clone()))
            .on_ok(move |_, _, cx| {
                handle.update(cx, |workspace, cx| {
                    if workspace.engine_generation() == generation {
                        workspace.run_action(id.clone(), action, cx)
                    }
                });
                true
            })
    });
}

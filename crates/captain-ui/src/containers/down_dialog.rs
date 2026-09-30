use captain_core::model::ProjectAction;
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use crate::widgets::danger_footer;
use crate::workspace::Workspace;

/// Asks before `docker compose down`, which removes the project's containers. Does
/// nothing if the workspace switched engines while the dialog was open.
pub fn open(project: String, handle: Entity<Workspace>, window: &mut Window, cx: &mut App) {
    let generation = handle.read(cx).engine_generation();
    let title = SharedString::from(format!("Down {project}?"));
    let description = SharedString::from(format!(
        "Stop and remove the containers of {project}? Volumes stay."
    ));
    window.open_alert_dialog(cx, move |alert, _, _| {
        let handle = handle.clone();
        let project = project.clone();
        alert
            .title(title.clone())
            .description(description.clone())
            .footer(danger_footer(
                "Down",
                format!("Stop and remove the containers of {project}. Volumes stay."),
            ))
            .on_ok(move |_, _, cx| {
                handle.update(cx, |workspace, cx| {
                    if workspace.engine_generation() == generation {
                        workspace.run_project_action(project.clone(), ProjectAction::Down, cx)
                    }
                });
                true
            })
    });
}

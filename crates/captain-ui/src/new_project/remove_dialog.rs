use gpui_kit::component::WindowExt;
use gpui_kit::*;

use captain_core::store::GroupKey;

use super::known_model;
use crate::widgets::{danger_footer, error_notification};
use crate::workspace::{Page, Workspace};

/// Asks before Captain forgets the known project `name`. Its files stay. When the
/// Project page shows it and nothing of it runs, the page goes to Containers.
pub fn open(
    name: String,
    folder: String,
    workspace: Entity<Workspace>,
    window: &mut Window,
    cx: &mut App,
) {
    let title = SharedString::from(format!("Remove {name} from Captain?"));
    let description = SharedString::from(format!(
        "Captain stops listing {name} when it is not running. The files in {folder} stay, and so do its containers and volumes."
    ));
    window.open_alert_dialog(cx, move |alert, _, _| {
        let (name, workspace) = (name.clone(), workspace.clone());
        alert
            .title(title.clone())
            .description(description.clone())
            .footer(danger_footer(
                "Remove",
                format!("Forget {name}. No file, container, or volume is deleted."),
            ))
            .on_ok(move |_, window, cx| {
                let removed = known_model(cx).update(cx, |model, cx| model.remove(&name, cx));
                if let Err(error) = removed {
                    window.push_notification(
                        error_notification(format!("Cannot remove {name}"), error),
                        cx,
                    );
                    return true;
                }
                workspace.update(cx, |workspace, cx| {
                    let shown = workspace.focus() == Some(&GroupKey::Project(name.clone()));
                    if shown && workspace.focused_group().is_none() {
                        workspace.set_page(Page::Containers, cx);
                    }
                });
                true
            })
    });
}

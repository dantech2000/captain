use captain_core::extension::InstalledExtension;
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::ExtensionsModel;
use crate::widgets::danger_footer;

/// Asks before removing an extension with its backend, files, and image.
pub fn open(
    model: Entity<ExtensionsModel>,
    extension: InstalledExtension,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_alert_dialog(cx, move |alert, _, _| {
        let (model, extension) = (model.clone(), extension.clone());
        alert
            .title(format!("Remove {}?", extension.title()))
            .description(
                "Captain closes its window, stops its backend and deletes the backend's \
                 volumes, deletes its files, and removes its image.",
            )
            .footer(danger_footer(
                "Remove",
                "Remove the extension and stop its backend.",
            ))
            .on_ok(move |_, _, cx| {
                let extension = extension.clone();
                model.update(cx, |model, cx| model.remove(extension, cx));
                true
            })
    });
}

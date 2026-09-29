use captain_core::extension::InstalledExtension;
use gpui_kit::component::WindowExt;
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::*;

use super::ExtensionsModel;

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
            .show_cancel(true)
            .ok_text("Remove")
            .ok_variant(ButtonVariant::Danger)
            .on_ok(move |_, _, cx| {
                let extension = extension.clone();
                model.update(cx, |model, cx| model.remove(extension, cx));
                true
            })
    });
}

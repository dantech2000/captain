use gpui_kit::component::WindowExt;
use gpui_kit::*;

use crate::engine_host::HostModel;
use crate::widgets::danger_footer;

/// Asks before Reset, which deletes the VM and everything in it.
pub fn open(model: Entity<HostModel>, window: &mut Window, cx: &mut App) {
    window.open_alert_dialog(cx, move |alert, _, _| {
        let model = model.clone();
        alert
            .title("Reset Captain Engine?")
            .description(
                "This deletes the VM and all its containers, images, and volumes. \
                 It cannot be undone. The next start sets up a new, empty engine.",
            )
            .footer(danger_footer(
                "Reset",
                "Delete Captain Engine with all its containers, images, and volumes.",
            ))
            .on_ok(move |_, _, cx| {
                model.update(cx, |model, cx| model.reset(cx));
                true
            })
    });
}

//! The question before the engine's disk grows, which cannot be undone.

use captain_core::HostResources;
use captain_core::format::bytes_label;
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::SettingsView;
use crate::engine_host::HostModel;
use crate::widgets::confirm_footer;

/// Asks before the disk grows to `disk` bytes. Grow saves it; a stopped engine gets
/// it at once and a running one on its next start. Either way the disk stepper
/// shows the saved size again.
pub fn open(
    view: WeakEntity<SettingsView>,
    model: Entity<HostModel>,
    disk: u64,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_alert_dialog(cx, move |alert, _, _| {
        let (model, done, cancelled, closed) =
            (model.clone(), view.clone(), view.clone(), view.clone());
        alert
            .title(format!("Grow the disk to {}?", bytes_label(disk)))
            .description(
                "A disk can grow but not shrink. A running engine gets the larger disk \
                 when it restarts.",
            )
            .footer(confirm_footer(
                "Grow",
                "Save the larger disk size. The disk cannot shrink back later.",
            ))
            .on_ok(move |_, _, cx| {
                model.update(cx, |model, cx| {
                    let grown = HostResources {
                        disk_bytes: disk,
                        ..model.resources()
                    };
                    model.set_resources(grown, cx);
                });
                clear(&done, cx);
                true
            })
            .on_cancel(move |_, _, cx| {
                clear(&cancelled, cx);
                true
            })
            .on_close(move |_, _, cx| clear(&closed, cx))
    });
}

/// Drops the size that waited for Grow.
fn clear(view: &WeakEntity<SettingsView>, cx: &mut App) {
    view.update(cx, |view, cx| {
        view.disk_pending = None;
        cx.notify();
    })
    .ok();
}

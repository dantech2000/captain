use captain_core::snapshot::Snapshot;
use gpui_kit::component::WindowExt;
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::*;

use super::SnapshotsModel;

/// Asks before deleting a snapshot. The engine keeps running.
pub fn open(model: Entity<SnapshotsModel>, snapshot: Snapshot, window: &mut Window, cx: &mut App) {
    window.open_alert_dialog(cx, move |alert, _, _| {
        let (model, snapshot) = (model.clone(), snapshot.clone());
        alert
            .title(format!("Delete \"{}\"?", snapshot.metadata.name))
            .description("This deletes the snapshot. It cannot be undone.")
            .show_cancel(true)
            .ok_text("Delete")
            .ok_variant(ButtonVariant::Danger)
            .on_ok(move |_, _, cx| {
                let snapshot = snapshot.clone();
                model.update(cx, |model, cx| model.delete(snapshot, cx));
                true
            })
    });
}

//! Raise Memory: the fix for a container that runs out of memory at its limit, as
//! `docker update --memory` does.

use captain_core::format::bytes_label;
use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::controller::TrayHandle;
use crate::window;

/// Gives container `id` a limit of `bytes`. The main window tells how it went when
/// it is open; a failure opens it.
pub fn raise_memory(id: String, name: String, bytes: u64, cx: &mut App) {
    let Some(engine) = window::workspace(cx).read(cx).engine() else {
        return;
    };
    // Noted now, so the menu does not offer the same raise again.
    note_raise(&id, false, cx);
    let update = engine.update_memory(&id, bytes);
    cx.spawn(async move |cx| {
        let result = update.await;
        cx.update(|cx| {
            let note = match result {
                Ok(()) => {
                    Notification::success(format!("{name} can now use {}.", bytes_label(bytes)))
                }
                Err(error) => {
                    tracing::warn!(%error, "cannot raise the memory limit");
                    note_raise(&id, true, cx);
                    window::show(cx);
                    captain_ui::error_notification(
                        "Raise memory failed",
                        format!("Captain could not raise the limit: {error}"),
                    )
                }
            };
            window::update_main(cx, |window, cx| window.push_notification(note, cx));
        });
    })
    .detach();
}

/// Notes a Raise Memory from the menu, or forgets one that `failed`, so the menu
/// offers it once per run, as the Project page does.
fn note_raise(id: &str, failed: bool, cx: &mut App) {
    if let Some(tray) = cx.try_global::<TrayHandle>().map(|handle| handle.0.clone()) {
        tray.update(cx, |tray, cx| {
            if failed {
                tray.exits.forget_raise(id);
            } else {
                tray.exits.record_raise(id);
            }
            tray.workspace_changed(cx);
        });
    }
}

//! What the menu items do when no view handles them: the app and window items,
//! About, Help, and Settings while the main window is closed or not in front.

use std::path::Path;

use captain_ui::{ShowSettings, diagnostics_model, open_about};
use gpui_kit::*;

use super::entries::{
    About, BringAllToFront, CloseWindow, Hide, HideOthers, Minimize, OpenGuide, OpenShortcuts,
    ReportIssue, ShowAll, ShowLogs, Zoom, menu_bindings, menu_specs, menus,
};
use crate::window;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const GUIDE: &str = concat!(
    env!("CARGO_PKG_REPOSITORY"),
    "/blob/main/docs/guide/README.md"
);
const SHORTCUTS: &str = concat!(
    env!("CARGO_PKG_REPOSITORY"),
    "/blob/main/docs/guide/getting-started.md#keyboard-shortcuts"
);

/// Binds the menu's own keys, adds the app-wide handlers, and sets the menus.
/// Call it after every other key binding: GPUI reads each item's shortcut from
/// the bindings when the menus are set.
pub fn register(cx: &mut App) {
    cx.bind_keys(menu_bindings());
    cx.on_action(|_: &About, cx| {
        cx.defer(|cx| {
            window::show(cx);
            let workspace = window::workspace(cx);
            window::update_main(cx, |window, cx| open_about(workspace, window, cx));
        })
    });
    // The shell handles Settings while the main window is in front.
    cx.on_action(|_: &ShowSettings, cx| {
        if !window::main_is_active(cx) {
            cx.defer(window::show_settings);
        }
    });
    cx.on_action(|_: &Hide, cx| cx.hide());
    cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
    cx.on_action(|_: &ShowAll, cx| cx.unhide_other_apps());
    cx.on_action(|_: &CloseWindow, cx| on_active(cx, |window| window.remove_window()));
    cx.on_action(|_: &Minimize, cx| on_active(cx, |window| window.minimize_window()));
    cx.on_action(|_: &Zoom, cx| on_active(cx, |window| window.zoom_window()));
    cx.on_action(|_: &BringAllToFront, cx| cx.defer(bring_all_to_front));
    cx.on_action(|_: &OpenGuide, cx| cx.open_url(GUIDE));
    cx.on_action(|_: &OpenShortcuts, cx| cx.open_url(SHORTCUTS));
    cx.on_action(|_: &ReportIssue, cx| cx.open_url(&format!("{REPOSITORY}/issues/new")));
    cx.on_action(|_: &ShowLogs, cx| {
        let dir =
            diagnostics_model(cx).and_then(|model| model.read(cx).log_dir().map(Path::to_path_buf));
        if let Some(dir) = dir {
            cx.open_with_system(&dir);
        }
    });
    cx.set_menus(menus(menu_specs()));
}

/// Runs `f` on the window in front. A menu action runs while GPUI updates that
/// window, so `f` waits until the update ends.
fn on_active(cx: &mut App, f: impl FnOnce(&mut Window) + 'static) {
    if let Some(handle) = cx.active_window() {
        cx.defer(move |cx| {
            handle.update(cx, |_, window, _| f(window)).ok();
        });
    }
}

/// Raises every Captain window, and then the one that was in front.
fn bring_all_to_front(cx: &mut App) {
    let active = cx.active_window();
    let others = cx
        .windows()
        .into_iter()
        .filter(|handle| Some(*handle) != active);
    for handle in others.chain(active).collect::<Vec<_>>() {
        handle
            .update(cx, |_, window, _| window.activate_window())
            .ok();
    }
}

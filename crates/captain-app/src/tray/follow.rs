//! Shows or hides the menu bar icon as the settings say. See feature 0015.

use gpui_kit::*;

use super::controller::{is_running, start, stop};

/// Starts the icon if the settings show it, and follows the switch from then on.
pub fn follow_settings(cx: &mut App) {
    apply(cx);
    captain_ui::observe_settings(cx, apply).detach();
}

fn apply(cx: &mut App) {
    let show = captain_ui::current_settings(cx).show_menu_bar_icon;
    match (show, is_running(cx)) {
        (true, false) => start(cx),
        (false, true) => stop(cx),
        _ => {}
    }
}

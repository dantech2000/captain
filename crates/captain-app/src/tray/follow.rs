//! Shows or hides the menu bar icon as the settings say. See feature 0015.

use gpui_kit::*;

use super::controller::{is_running, start, stop};
use super::icon_view::set_status_dot;

/// Starts the icon if the settings show it, and follows the switch from then on.
pub fn follow_settings(cx: &mut App) {
    apply(cx);
    captain_ui::observe_settings(cx, apply).detach();
}

fn apply(cx: &mut App) {
    let settings = captain_ui::current_settings(cx);
    match (settings.show_menu_bar_icon, is_running(cx)) {
        (true, false) => start(cx),
        (false, true) => stop(cx),
        (true, true) => set_status_dot(settings.menu_bar_status_dot, cx),
        (false, false) => {}
    }
}

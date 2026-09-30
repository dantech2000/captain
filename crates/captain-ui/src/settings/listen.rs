//! Click handlers that reach the Settings page from a sheet or a menu, where
//! `cx.listener` of the page is not at hand.

use gpui_kit::*;

use super::SettingsView;

/// A click handler that runs `f` on the page, if it is still open.
pub fn listen(
    view: &WeakEntity<SettingsView>,
    f: impl Fn(&mut SettingsView, &mut Window, &mut Context<SettingsView>) + 'static,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    let view = view.clone();
    move |_, window, cx| {
        view.update(cx, |view, cx| f(view, window, cx)).ok();
    }
}

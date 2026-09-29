//! App-wide actions, key bindings, and the menu bar.

use gpui_kit::*;

gpui_kit::actions!(captain, [Quit]);

pub fn register(cx: &mut App) {
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.bind_keys([
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("ctrl-q", Quit, None),
    ]);
    cx.set_menus([Menu::new("Captain").items([MenuItem::action("Quit Captain", Quit)])]);

    // Closing the last window quits, on every platform.
    cx.on_window_closed(|cx, _| {
        if cx.windows().is_empty() {
            cx.quit();
        }
    })
    .detach();
}

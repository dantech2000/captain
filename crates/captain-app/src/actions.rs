//! App-wide actions, key bindings, and the menu bar.

use captain_ui::ToggleCommandPalette;
use gpui_kit::*;

use crate::quit;

gpui_kit::actions!(captain, [Quit]);

pub fn register(cx: &mut App) {
    cx.on_action(|_: &Quit, cx| quit::quit(cx));
    cx.bind_keys([
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("ctrl-q", Quit, None),
        KeyBinding::new("cmd-k", ToggleCommandPalette, None),
        KeyBinding::new("ctrl-k", ToggleCommandPalette, None),
    ]);
    cx.set_menus([Menu::new("Captain").items([MenuItem::action("Quit Captain", Quit)])]);

    // Closing the last window keeps Captain running while its menu bar icon is up;
    // Quit exits. Without the icon (on Linux, or if it failed), closing the last
    // window quits, on every platform. See ADR 0006.
    cx.set_quit_mode(QuitMode::Explicit);
    cx.on_window_closed(|cx, _| {
        if cx.windows().is_empty() && !has_tray(cx) {
            quit::quit(cx);
        }
    })
    .detach();
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn has_tray(cx: &App) -> bool {
    crate::tray::is_running(cx)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn has_tray(_: &App) -> bool {
    false
}

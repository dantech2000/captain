//! The panel's actions and keys. ⌃` shows or hides the panel everywhere, as in Zed
//! (<https://github.com/zed-industries/zed/blob/main/assets/keymaps/default-macos.json>).
//! While a terminal has focus, ⌘T opens a tab and ⌘W closes one on macOS. Linux
//! and Windows use Ctrl-Shift-T and Ctrl-Shift-W, because Ctrl-T and Ctrl-W
//! belong to the shell.

use gpui_kit::*;

gpui_kit::actions!(captain, [ToggleTerminal, NewTerminalTab, CloseTerminalTab]);

/// The key context of the panel, around every tab's grid.
pub const PANEL_CONTEXT: &str = "CaptainTerminalPanel";

const MACOS: bool = cfg!(target_os = "macos");

/// The keys of [`ToggleTerminal`], as the status bar's key chips show them.
pub const TOGGLE_KEYS: &[&str] = if MACOS { &["⌃", "`"] } else { &["Ctrl", "`"] };

/// The keys of [`NewTerminalTab`].
pub const NEW_TAB_KEYS: &[&str] = if MACOS {
    &["⌘", "T"]
} else {
    &["Ctrl", "Shift", "T"]
};

/// The keys of [`CloseTerminalTab`].
pub const CLOSE_TAB_KEYS: &[&str] = if MACOS {
    &["⌘", "W"]
} else {
    &["Ctrl", "Shift", "W"]
};

/// The key bindings, for the app to register.
pub fn terminal_bindings() -> Vec<KeyBinding> {
    let (new_tab, close_tab) = if MACOS {
        ("cmd-t", "cmd-w")
    } else {
        ("ctrl-shift-t", "ctrl-shift-w")
    };
    let panel = Some(PANEL_CONTEXT);
    vec![
        KeyBinding::new("ctrl-`", ToggleTerminal, None),
        KeyBinding::new(new_tab, NewTerminalTab, panel),
        KeyBinding::new(close_tab, CloseTerminalTab, panel),
    ]
}

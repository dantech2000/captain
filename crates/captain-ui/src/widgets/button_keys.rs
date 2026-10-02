//! Enter and Space on a focused button. GPUI turns them into a click on key up,
//! but only when no key binding takes the key down first, and a kit Dialog binds
//! Enter to Confirm. So Tab to a dialog's Restore button and Enter closed the
//! dialog without restoring. A `NoAction` binding in a deeper key context hides
//! the dialog's binding while a button has focus, and the click goes through.
//! See the keymap's precedence rules:
//! https://github.com/zed-industries/zed/blob/main/crates/gpui/src/keymap.rs

use gpui_kit::*;

/// The key context around each focusable Captain button.
pub const BUTTON_CONTEXT: &str = "CaptainButton";

/// The key bindings, for the app to register.
pub fn button_bindings() -> Vec<KeyBinding> {
    let button = Some(BUTTON_CONTEXT);
    vec![
        KeyBinding::new("enter", NoAction, button),
        KeyBinding::new("space", NoAction, button),
    ]
}

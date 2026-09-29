use gpui_kit::*;

/// The palette's key context. Its bindings apply only while the palette has focus.
pub const CONTEXT: &str = "CommandPalette";

/// The search field binds the same keys in its own context. A binding in a deeper
/// context wins, so the palette binds them again for its search field.
const SEARCH_FIELD: &str = "CommandPalette > Input";

gpui_kit::actions!(command_palette, [SelectPrev, SelectNext, Confirm, Dismiss]);

/// Binds the palette keys: up and down move the highlight, enter runs, escape closes.
pub fn init(cx: &mut App) {
    let mut bindings = Vec::new();
    for context in [CONTEXT, SEARCH_FIELD] {
        bindings.extend([
            KeyBinding::new("up", SelectPrev, Some(context)),
            KeyBinding::new("down", SelectNext, Some(context)),
            KeyBinding::new("enter", Confirm, Some(context)),
            KeyBinding::new("escape", Dismiss, Some(context)),
        ]);
    }
    cx.bind_keys(bindings);
}

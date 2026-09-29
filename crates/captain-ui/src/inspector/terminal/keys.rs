use gpui_kit::*;

/// The key context of the focused grid.
pub const KEY_CONTEXT: &str = "CaptainTerminal";

/// Marks that [`bind`] ran, so the bindings go in once.
struct Bound;

impl Global for Bound {}

/// Frees Ctrl-K and Ctrl-Q inside the terminal. The app binds them to the command
/// palette and Quit, but shells use them (Ctrl-K cuts to the end of the line). A
/// `NoAction` binding in the terminal's context hides the app binding there, so the
/// keystroke reaches the grid. Cmd-K and Cmd-Q on macOS still work.
pub fn bind(cx: &mut App) {
    if cx.has_global::<Bound>() {
        return;
    }
    cx.set_global(Bound);
    cx.bind_keys([
        KeyBinding::new("ctrl-k", NoAction, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-q", NoAction, Some(KEY_CONTEXT)),
    ]);
}

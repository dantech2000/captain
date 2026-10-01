use gpui_kit::*;

/// The key context of the focused grid.
pub const KEY_CONTEXT: &str = "CaptainTerminal";

/// Marks that [`bind`] ran, so the bindings go in once.
struct Bound;

impl Global for Bound {}

/// Frees Ctrl-K, Ctrl-Q, and Ctrl-B inside the terminal. The app binds them to the
/// command palette, Quit, and the sidebar, but shells use them (Ctrl-K cuts to the
/// end of the line, Ctrl-B moves back and is the tmux prefix). A `NoAction` binding
/// in the terminal's context hides the app binding there, so the keystroke reaches
/// the grid. Cmd-K, Cmd-Q, and Cmd-B on macOS still work.
pub fn bind(cx: &mut App) {
    if cx.has_global::<Bound>() {
        return;
    }
    cx.set_global(Bound);
    cx.bind_keys([
        KeyBinding::new("ctrl-k", NoAction, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-q", NoAction, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-b", NoAction, Some(KEY_CONTEXT)),
    ]);
}

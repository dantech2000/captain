use gpui_kit::*;

/// The key context of the focused grid.
pub const KEY_CONTEXT: &str = "CaptainTerminal";

/// The Ctrl shortcuts of the app that shells need: Ctrl-K cuts to the end of the
/// line, Ctrl-B moves back and is the tmux prefix, Ctrl-N is the next history line,
/// Ctrl-Q resumes output, and Ctrl-1 to Ctrl-9 and Ctrl-, are keys for programs
/// such as tmux and Emacs.
const FREED: [&str; 14] = [
    "ctrl-k", "ctrl-q", "ctrl-b", "ctrl-n", "ctrl-1", "ctrl-2", "ctrl-3", "ctrl-4", "ctrl-5",
    "ctrl-6", "ctrl-7", "ctrl-8", "ctrl-9", "ctrl-,",
];

/// Marks that [`bind`] ran, so the bindings go in once.
struct Bound;

impl Global for Bound {}

/// Frees [`FREED`] inside the terminal. The app binds them to the command palette,
/// Quit, the sidebar, the New sheet, and the pages. A `NoAction` binding in the
/// terminal's context hides the app binding there, so the keystroke reaches the
/// grid. The ⌘ shortcuts on macOS still work.
pub fn bind(cx: &mut App) {
    if cx.has_global::<Bound>() {
        return;
    }
    cx.set_global(Bound);
    cx.bind_keys(FREED.map(|keystroke| KeyBinding::new(keystroke, NoAction, Some(KEY_CONTEXT))));
}

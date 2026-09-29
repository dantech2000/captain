use gpui_kit::*;

/// The key context of the Files tab. Its bindings apply while the tab has focus.
pub const KEY_CONTEXT: &str = "CaptainFiles";

gpui_kit::actions!(
    captain_files,
    [SelectPrev, SelectNext, OpenSelected, GoUp, ClosePreview]
);

/// Marks that [`bind`] ran, so the bindings go in once.
struct Bound;

impl Global for Bound {}

/// Up and Down move the selection, Enter opens it, and Backspace or Cmd-Up (Alt-Up
/// on Windows and Linux) goes to the parent folder or closes the preview. Escape
/// closes the preview.
pub fn bind(cx: &mut App) {
    if cx.has_global::<Bound>() {
        return;
    }
    cx.set_global(Bound);
    let context = Some(KEY_CONTEXT);
    cx.bind_keys([
        KeyBinding::new("up", SelectPrev, context),
        KeyBinding::new("down", SelectNext, context),
        KeyBinding::new("enter", OpenSelected, context),
        KeyBinding::new("backspace", GoUp, context),
        KeyBinding::new("escape", ClosePreview, context),
        KeyBinding::new(
            if cfg!(target_os = "macos") {
                "cmd-up"
            } else {
                "alt-up"
            },
            GoUp,
            context,
        ),
    ]);
}

use gpui_kit::*;

use super::hover_help;

/// Groups the hovers of each mouse move, so a control inside another shows its own
/// hint (see [`HoverHelp`](super::hover_help::HoverHelp)). Put it first in a
/// window's root: its capture listener then runs before every hover listener, and
/// its bubble listener after them. After each frame with a hint, it drops the
/// hints of elements that left the screen.
pub fn hover_batch() -> impl IntoElement {
    canvas(
        |_, _, _| {},
        |_, _, window, cx| {
            // A hint can outlive its element; look after the frame is on screen.
            if hover_help(cx).is_some_and(|model| model.read(cx).hint().is_some()) {
                window.on_next_frame(|_, cx| forget_gone(cx));
            }
            window.on_mouse_event(|_: &MouseMoveEvent, phase, _, cx| {
                let Some(model) = hover_help(cx) else {
                    return;
                };
                model.update(cx, |help, cx| {
                    let changed = match phase {
                        DispatchPhase::Capture => help.begin_batch(),
                        DispatchPhase::Bubble => help.end_batch(),
                    };
                    if changed {
                        cx.notify();
                    }
                });
            });
        },
    )
    .absolute()
    .size_0()
}

fn forget_gone(cx: &mut App) {
    if let Some(model) = hover_help(cx) {
        model.update(cx, |help, cx| {
            if help.forget_gone() {
                cx.notify();
            }
        });
    }
}

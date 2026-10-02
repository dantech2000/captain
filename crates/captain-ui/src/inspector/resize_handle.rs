//! The inspector's left edge: drag it to make the panel wider or narrower, and
//! double-click it to go back to the default width.

use gpui_kit::*;

use crate::help::HelpExt;
use crate::theme::Palette;

/// The width the inspector starts with.
pub const DEFAULT_WIDTH: f32 = 400.;
const MIN_WIDTH: f32 = 320.;
const MAX_WIDTH: f32 = 760.;
/// What the page beside the panel needs, so the Containers table keeps its name
/// column and its fixed columns.
const PAGE_MIN: f32 = 700.;

/// What a drag of the edge carries. It draws nothing under the mouse.
pub struct DraggedEdge;

impl Render for DraggedEdge {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

/// The width the panel gets when it `wants` a width: from a drag of its edge, or
/// the width it keeps while the window or the sidebar changes. `room` is the width
/// the page and the panel share, the window less the rail and the projects list;
/// the page keeps enough of it for its table.
pub fn width_for(wants: f32, room: f32) -> f32 {
    let max = (room - PAGE_MIN).clamp(MIN_WIDTH, MAX_WIDTH);
    wants.clamp(MIN_WIDTH, max)
}

/// A strip over the panel's left border. `on_reset` runs on a double-click.
pub fn render(
    palette: &Palette,
    on_reset: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let hover = palette.accent_fg;
    div()
        .id("inspector-resize")
        .absolute()
        .top_0()
        .bottom_0()
        .left(px(-3.))
        .w(px(6.))
        .cursor(CursorStyle::ResizeLeftRight)
        .hover(move |style| style.bg(hover.opacity(0.4)))
        .on_drag(DraggedEdge, |_, _, _, cx| cx.new(|_| DraggedEdge))
        .on_click(move |event, window, cx| {
            if event.click_count() == 2 {
                on_reset(window, cx);
            }
        })
        .help("Drag to resize the details panel. Double-click to reset its width.")
}

#[cfg(test)]
mod tests;

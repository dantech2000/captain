//! The panel's top edge: drag it to make the panel taller or shorter, and
//! double-click it to go back to the default height. It works like the
//! inspector's left edge (`inspector::resize_handle`).

use gpui_kit::*;

use crate::help::HelpExt;
use crate::theme::Palette;

/// The height the panel starts with.
pub const DEFAULT_HEIGHT: f32 = 280.;
const MIN_HEIGHT: f32 = 120.;
/// What the page above the panel keeps, so its header and a few rows still show.
const PAGE_MIN: f32 = 240.;

/// What a drag of the edge carries. It draws nothing under the mouse.
pub struct DraggedPanelEdge;

impl Render for DraggedPanelEdge {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

/// The tallest the panel may be in a window `room` high.
pub fn max_height(room: f32) -> f32 {
    (room - PAGE_MIN).max(MIN_HEIGHT)
}

/// The height for a mouse at `mouse_y` while the panel's bottom is at `bottom`, in
/// a window `room` high.
pub fn height_for(bottom: Pixels, mouse_y: Pixels, room: f32) -> f32 {
    f32::from(bottom - mouse_y).clamp(MIN_HEIGHT, max_height(room))
}

/// A strip over the panel's top border. `on_reset` runs on a double-click.
pub fn render(
    palette: &Palette,
    on_reset: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let hover = palette.accent_fg;
    div()
        .id("terminal-panel-resize")
        .absolute()
        .left_0()
        .right_0()
        .top(px(-3.))
        .h(px(6.))
        .cursor(CursorStyle::ResizeUpDown)
        .hover(move |style| style.bg(hover.opacity(0.4)))
        .on_drag(DraggedPanelEdge, |_, _, _, cx| cx.new(|_| DraggedPanelEdge))
        .on_click(move |event, window, cx| {
            if event.click_count() == 2 {
                on_reset(window, cx);
            }
        })
        .help("Drag to resize the terminal panel. Double-click to reset its height.")
}

#[cfg(test)]
mod tests;

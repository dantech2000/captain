//! The terminal grid element. Each frame it measures the cells, tells the pane the
//! grid size, and paints the pane's screen: backgrounds, then text runs per row, then
//! the cursor.

mod frame;
mod spans;

use gpui_kit::*;

use super::TerminalPane;
use super::colors::TerminalColors;
use super::metrics::GridMetrics;
use frame::Frame;

/// The text size of the terminal, and the height of one row.
pub const FONT_SIZE: Pixels = px(12.);
pub const LINE_HEIGHT: Pixels = px(16.);

pub struct TerminalGrid {
    pane: Entity<TerminalPane>,
    colors: TerminalColors,
    font: Font,
    focused: bool,
}

impl TerminalGrid {
    pub fn new(
        pane: Entity<TerminalPane>,
        colors: TerminalColors,
        family: SharedString,
        focused: bool,
    ) -> Self {
        Self {
            pane,
            colors,
            font: font(family),
            focused,
        }
    }

    /// The size of one cell in the monospace font.
    fn cell_size(&self, window: &Window) -> Size<Pixels> {
        let text = window.text_system();
        let font_id = text.resolve_font(&self.font);
        let width = text
            .advance(font_id, FONT_SIZE, 'm')
            .map(|advance| advance.width)
            .unwrap_or(FONT_SIZE * 0.6);
        size(width, LINE_HEIGHT)
    }
}

impl IntoElement for TerminalGrid {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TerminalGrid {
    type RequestLayoutState = ();
    type PrepaintState = Frame;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Frame {
        let metrics = GridMetrics::fit(bounds, self.cell_size(window));
        self.pane.update(cx, |pane, cx| pane.sync_size(metrics, cx));
        let screen = self.pane.read(cx).snapshot();
        let style = frame::Style {
            colors: &self.colors,
            font: &self.font,
            font_size: FONT_SIZE,
            focused: self.focused,
        };
        frame::build(&screen, &metrics, &style, window)
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        frame: &mut Frame,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for quad in frame.quads.drain(..) {
                window.paint_quad(quad);
            }
            for (origin, line) in &frame.lines {
                line.paint(*origin, LINE_HEIGHT, TextAlign::Left, None, window, cx)
                    .ok();
            }
            for quad in frame.cursor.drain(..) {
                window.paint_quad(quad);
            }
            if let Some((origin, line)) = &frame.cursor_text {
                line.paint(*origin, LINE_HEIGHT, TextAlign::Left, None, window, cx)
                    .ok();
            }
        });
    }
}

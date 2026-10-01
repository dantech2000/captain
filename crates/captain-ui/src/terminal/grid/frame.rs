//! Turns a [`Screen`] into what the grid paints: rectangles, shaped text, and the cursor.

use captain_terminal::{Cell, CursorShape, Screen};
use gpui_kit::*;

use super::spans::{spans_by, text_segments};
use crate::terminal::colors::TerminalColors;
use crate::terminal::metrics::GridMetrics;

/// The width of a beam cursor and the height of an underline cursor.
const CURSOR_BAR: Pixels = px(2.);

/// One frame of the grid, ready to paint.
pub struct Frame {
    pub quads: Vec<PaintQuad>,
    pub lines: Vec<(Point<Pixels>, ShapedLine)>,
    /// Quads and text for the cursor, painted last.
    pub cursor: Vec<PaintQuad>,
    pub cursor_text: Option<(Point<Pixels>, ShapedLine)>,
}

/// Shared inputs for building a frame.
pub struct Style<'a> {
    pub colors: &'a TerminalColors,
    pub font: &'a Font,
    pub font_size: Pixels,
    pub focused: bool,
}

pub fn build(screen: &Screen, metrics: &GridMetrics, style: &Style, window: &Window) -> Frame {
    let mut frame = Frame {
        quads: Vec::new(),
        lines: Vec::new(),
        cursor: Vec::new(),
        cursor_text: None,
    };
    let colors = style.colors;
    for (row, cells) in screen.lines.iter().enumerate() {
        let backgrounds = spans_by(cells, |cell| {
            let (_, bg) = cell_colors(cell, colors);
            (bg != colors.background).then_some(bg)
        });
        let selected = spans_by(cells, |cell| cell.flags.selected.then_some(()));
        for (range, bg) in backgrounds {
            let bounds = metrics.span(row, range.start, range.len());
            frame.quads.push(fill(bounds, bg));
        }
        for (range, ()) in selected {
            let bounds = metrics.span(row, range.start, range.len());
            frame.quads.push(fill(bounds, colors.selection));
        }
        for range in text_segments(cells) {
            let cells = &cells[range.clone()];
            let width = if cells[0].flags.wide { 2. } else { 1. };
            let force = metrics.cell.width * width;
            let line = shape(cells, None, style, force, window);
            frame
                .lines
                .push((metrics.cell_origin(row, range.start), line));
        }
    }
    if let Some(cursor) = cursor(screen, metrics, style, window) {
        frame.cursor = cursor.0;
        frame.cursor_text = cursor.1;
    }
    frame
}

/// The text and background color of a cell, after inverse, dim, and hidden.
fn cell_colors(cell: &Cell, colors: &TerminalColors) -> (Hsla, Hsla) {
    let mut fg = colors.resolve(cell.fg);
    let mut bg = colors.resolve(cell.bg);
    if cell.flags.inverse {
        std::mem::swap(&mut fg, &mut bg);
    }
    if cell.flags.dim {
        fg = fg.alpha(fg.a * 0.6);
    }
    if cell.flags.hidden {
        fg = bg;
    }
    (fg, bg)
}

/// Shapes `cells` as one line, each glyph forced to `force_width`. `color` overrides
/// the text color, for the character under a block cursor.
fn shape(
    cells: &[Cell],
    color: Option<Hsla>,
    style: &Style,
    force_width: Pixels,
    window: &Window,
) -> ShapedLine {
    let mut text = String::new();
    let mut runs: Vec<TextRun> = Vec::new();
    for cell in cells {
        let (fg, _) = cell_colors(cell, style.colors);
        let color = color.unwrap_or(fg);
        let mut font = style.font.clone();
        if cell.flags.bold {
            font.weight = FontWeight::BOLD;
        }
        if cell.flags.italic {
            font.style = FontStyle::Italic;
        }
        let run = TextRun {
            len: cell.c.len_utf8(),
            font,
            color,
            background_color: None,
            underline: cell.flags.underline.then(|| UnderlineStyle {
                thickness: px(1.),
                color: Some(color),
                wavy: false,
            }),
            strikethrough: cell.flags.strikeout.then(|| StrikethroughStyle {
                thickness: px(1.),
                color: Some(color),
            }),
        };
        text.push(cell.c);
        match runs.last_mut() {
            Some(last)
                if TextRun {
                    len: last.len,
                    ..run.clone()
                } == *last =>
            {
                last.len += run.len;
            }
            _ => runs.push(run),
        }
    }
    window
        .text_system()
        .shape_line(text.into(), style.font_size, &runs, Some(force_width))
}

type CursorPaint = (Vec<PaintQuad>, Option<(Point<Pixels>, ShapedLine)>);

fn cursor(
    screen: &Screen,
    metrics: &GridMetrics,
    style: &Style,
    window: &Window,
) -> Option<CursorPaint> {
    let cursor = screen.cursor;
    if !cursor.visible {
        return None;
    }
    let cell = screen.lines.get(cursor.row)?.get(cursor.col)?;
    let width = if cell.flags.wide { 2 } else { 1 };
    let bounds = metrics.span(cursor.row, cursor.col, width);
    let color = style.colors.cursor;
    let kind = if style.focused {
        cursor.shape
    } else {
        CursorShape::HollowBlock
    };
    let paint = match kind {
        CursorShape::Block => {
            let text = (cell.c != ' ').then(|| {
                let force = metrics.cell.width * width as f32;
                let line = shape_cursor_char(cell, style, force, window);
                (bounds.origin, line)
            });
            (vec![fill(bounds, color)], text)
        }
        CursorShape::HollowBlock => (vec![outline(bounds, color, BorderStyle::Solid)], None),
        CursorShape::Beam => {
            let bar = Bounds::new(bounds.origin, size(CURSOR_BAR, bounds.size.height));
            (vec![fill(bar, color)], None)
        }
        CursorShape::Underline => {
            let top = bounds.origin.y + bounds.size.height - CURSOR_BAR;
            let bar = Bounds::new(
                point(bounds.origin.x, top),
                size(bounds.size.width, CURSOR_BAR),
            );
            (vec![fill(bar, color)], None)
        }
    };
    Some(paint)
}

fn shape_cursor_char(cell: &Cell, style: &Style, force: Pixels, window: &Window) -> ShapedLine {
    shape(
        std::slice::from_ref(cell),
        Some(style.colors.background),
        style,
        force,
        window,
    )
}

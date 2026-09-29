//! Converts alacritty's grid into a [`Screen`].

use alacritty_terminal::Term;
use alacritty_terminal::event::EventListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::color::Colors;
use alacritty_terminal::vte::ansi::{self, NamedColor};

use crate::{Cell, CellFlags, Color, Cursor, CursorShape, Rgb, Screen};

pub fn screen<T: EventListener>(term: &Term<T>) -> Screen {
    let content = term.renderable_content();
    let cols = term.columns();
    let rows = term.screen_lines();
    let offset = content.display_offset;
    let mut lines = vec![vec![Cell::default(); cols]; rows];

    for indexed in content.display_iter {
        let Some(row) = viewport_row(indexed.point.line.0, offset, rows) else {
            continue;
        };
        let col = indexed.point.column.0;
        if col >= cols {
            continue;
        }
        let source = &indexed.cell;
        let mut flags = cell_flags(source.flags);
        flags.selected = content
            .selection
            .is_some_and(|selection| selection.contains(indexed.point));
        let dim_named = matches!(source.fg, ansi::Color::Named(named) if is_dim(named));
        flags.dim |= dim_named;
        lines[row][col] = Cell {
            c: if flags.wide_spacer { ' ' } else { source.c },
            fg: color(source.fg, content.colors),
            bg: color(source.bg, content.colors),
            flags,
        };
    }

    let point = content.cursor.point;
    let shape = cursor_shape(content.cursor.shape);
    let row = viewport_row(point.line.0, offset, rows);
    let cursor = Cursor {
        row: row.unwrap_or(0),
        col: point.column.0.min(cols.saturating_sub(1)),
        shape: shape.unwrap_or_default(),
        visible: shape.is_some() && row.is_some(),
    };

    Screen {
        cols,
        rows,
        lines,
        cursor,
        display_offset: offset,
        history: term.grid().history_size(),
    }
}

/// The view row of a grid line, or `None` if it is outside the view.
fn viewport_row(line: i32, offset: usize, rows: usize) -> Option<usize> {
    usize::try_from(line + offset as i32)
        .ok()
        .filter(|row| *row < rows)
}

fn cell_flags(flags: Flags) -> CellFlags {
    CellFlags {
        bold: flags.contains(Flags::BOLD),
        italic: flags.contains(Flags::ITALIC),
        underline: flags.intersects(Flags::ALL_UNDERLINES),
        inverse: flags.contains(Flags::INVERSE),
        dim: flags.contains(Flags::DIM),
        strikeout: flags.contains(Flags::STRIKEOUT),
        hidden: flags.contains(Flags::HIDDEN),
        wide: flags.contains(Flags::WIDE_CHAR),
        wide_spacer: flags.contains(Flags::WIDE_CHAR_SPACER),
        selected: false,
    }
}

/// Our color for an alacritty color. A palette entry the program redefined (OSC 4, 10,
/// or 11) becomes its RGB value.
fn color(color: ansi::Color, colors: &Colors) -> Color {
    match color {
        ansi::Color::Spec(rgb) => Color::Rgb(Rgb::new(rgb.r, rgb.g, rgb.b)),
        ansi::Color::Indexed(index) => {
            overridden(colors, usize::from(index)).unwrap_or(Color::Indexed(index))
        }
        ansi::Color::Named(named) => {
            if let Some(color) = overridden(colors, named as usize) {
                return color;
            }
            match named {
                NamedColor::Foreground
                | NamedColor::BrightForeground
                | NamedColor::DimForeground
                | NamedColor::Cursor => Color::Foreground,
                NamedColor::Background => Color::Background,
                other => Color::Indexed(ansi_index(other)),
            }
        }
    }
}

fn overridden(colors: &Colors, index: usize) -> Option<Color> {
    colors[index].map(|rgb| Color::Rgb(Rgb::new(rgb.r, rgb.g, rgb.b)))
}

/// The ANSI index (0 to 15) of a named color. Dim colors map to their normal color;
/// the dim flag carries the rest.
fn ansi_index(named: NamedColor) -> u8 {
    let index = named as usize;
    if index < 16 {
        index as u8
    } else {
        (index - NamedColor::DimBlack as usize).min(7) as u8
    }
}

fn is_dim(named: NamedColor) -> bool {
    let index = named as usize;
    (NamedColor::DimBlack as usize..=NamedColor::DimWhite as usize).contains(&index)
        || named == NamedColor::DimForeground
}

fn cursor_shape(shape: ansi::CursorShape) -> Option<CursorShape> {
    match shape {
        ansi::CursorShape::Block => Some(CursorShape::Block),
        ansi::CursorShape::Underline => Some(CursorShape::Underline),
        ansi::CursorShape::Beam => Some(CursorShape::Beam),
        ansi::CursorShape::HollowBlock => Some(CursorShape::HollowBlock),
        ansi::CursorShape::Hidden => None,
    }
}

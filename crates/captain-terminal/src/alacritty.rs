//! The [`Emulator`] on `alacritty_terminal`. Alacritty types stay in this module.

mod convert;
mod listener;

use alacritty_terminal::Term;
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::selection::{Selection, SelectionType};
use alacritty_terminal::term::{Config, TermMode};
use alacritty_terminal::vte::ansi::Processor;

use crate::{Emulator, GridPoint, InputModes, Screen, SelectionKind, Side};
use listener::Listener;

/// Lines of history kept above the live screen.
const HISTORY: usize = 10_000;

/// An emulator backed by alacritty's `Term` and the `vte` parser.
pub struct AlacrittyEmulator {
    term: Term<Listener>,
    parser: Processor,
    listener: Listener,
}

impl AlacrittyEmulator {
    pub fn new(cols: u16, rows: u16) -> Self {
        let listener = Listener::default();
        let config = Config {
            scrolling_history: HISTORY,
            ..Config::default()
        };
        let term = Term::new(config, &GridSize::new(cols, rows), listener.clone());
        Self {
            term,
            parser: Processor::new(),
            listener,
        }
    }

    /// A viewport point as a point in alacritty's grid, where line 0 is the top of the
    /// live screen and history lines are negative.
    fn grid_point(&self, point: GridPoint) -> (Point, alacritty_terminal::index::Side) {
        let offset = self.term.grid().display_offset() as i32;
        let rows = self.term.screen_lines().max(1);
        let cols = self.term.columns().max(1);
        let row = point.row.min(rows - 1) as i32;
        let col = point.col.min(cols - 1);
        let side = match point.side {
            Side::Left => alacritty_terminal::index::Side::Left,
            Side::Right => alacritty_terminal::index::Side::Right,
        };
        (Point::new(Line(row - offset), Column(col)), side)
    }
}

impl Emulator for AlacrittyEmulator {
    fn feed(&mut self, bytes: &[u8]) {
        self.parser.advance(&mut self.term, bytes);
    }

    fn resize(&mut self, cols: u16, rows: u16) {
        self.term.resize(GridSize::new(cols, rows));
    }

    fn size(&self) -> (u16, u16) {
        let cols = u16::try_from(self.term.columns()).unwrap_or(u16::MAX);
        let rows = u16::try_from(self.term.screen_lines()).unwrap_or(u16::MAX);
        (cols, rows)
    }

    fn snapshot(&self) -> Screen {
        convert::screen(&self.term)
    }

    fn scroll(&mut self, lines: i32) {
        self.term.scroll_display(Scroll::Delta(lines));
    }

    fn scroll_to_bottom(&mut self) {
        self.term.scroll_display(Scroll::Bottom);
    }

    fn modes(&self) -> InputModes {
        let mode = self.term.mode();
        InputModes {
            app_cursor: mode.contains(TermMode::APP_CURSOR),
            bracketed_paste: mode.contains(TermMode::BRACKETED_PASTE),
            newline: mode.contains(TermMode::LINE_FEED_NEW_LINE),
            alt_screen: mode.contains(TermMode::ALT_SCREEN),
            alternate_scroll: mode.contains(TermMode::ALTERNATE_SCROLL),
        }
    }

    fn title(&self) -> Option<String> {
        self.listener.title()
    }

    fn take_replies(&mut self) -> Vec<u8> {
        self.listener.take_replies()
    }

    fn select_start(&mut self, point: GridPoint, kind: SelectionKind) {
        let (point, side) = self.grid_point(point);
        let kind = match kind {
            SelectionKind::Simple => SelectionType::Simple,
            SelectionKind::Word => SelectionType::Semantic,
            SelectionKind::Line => SelectionType::Lines,
        };
        self.term.selection = Some(Selection::new(kind, point, side));
    }

    fn select_update(&mut self, point: GridPoint) {
        let (point, side) = self.grid_point(point);
        if let Some(selection) = self.term.selection.as_mut() {
            selection.update(point, side);
        }
    }

    fn select_clear(&mut self) {
        self.term.selection = None;
    }

    fn selection_text(&self) -> Option<String> {
        self.term
            .selection_to_string()
            .filter(|text| !text.is_empty())
    }
}

/// A grid size in the form alacritty's `Term` takes.
struct GridSize {
    cols: usize,
    rows: usize,
}

impl GridSize {
    fn new(cols: u16, rows: u16) -> Self {
        Self {
            cols: usize::from(cols.max(2)),
            rows: usize::from(rows.max(1)),
        }
    }
}

impl Dimensions for GridSize {
    fn total_lines(&self) -> usize {
        self.rows
    }

    fn screen_lines(&self) -> usize {
        self.rows
    }

    fn columns(&self) -> usize {
        self.cols
    }
}

#[cfg(test)]
mod tests;

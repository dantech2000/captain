use crate::Color;

/// What the view needs to draw one frame: the visible rows, the cursor, and how far
/// the view is scrolled back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screen {
    pub cols: usize,
    pub rows: usize,
    /// `rows` rows of `cols` cells each, top to bottom.
    pub lines: Vec<Vec<Cell>>,
    pub cursor: Cursor,
    /// How many lines the view is scrolled back into the history. 0 is the live screen.
    pub display_offset: usize,
    /// How many lines of history there are above the live screen.
    pub history: usize,
}

impl Screen {
    /// The text of one row, without trailing spaces. Handy in tests.
    pub fn row_text(&self, row: usize) -> String {
        let text: String = self.lines[row]
            .iter()
            .filter(|cell| !cell.flags.wide_spacer)
            .map(|cell| cell.c)
            .collect();
        text.trim_end().to_string()
    }
}

/// One character cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
    pub flags: CellFlags,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: Color::Foreground,
            bg: Color::Background,
            flags: CellFlags::default(),
        }
    }
}

/// How a cell is drawn. `inverse` swaps the text and background colors; the view does
/// the swap, so the colors in [`Cell`] are the ones the program set.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CellFlags {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
    pub dim: bool,
    pub strikeout: bool,
    pub hidden: bool,
    /// The cell holds a character two cells wide.
    pub wide: bool,
    /// The empty right half of a wide character. The view draws nothing here.
    pub wide_spacer: bool,
    /// The cell is inside the selection.
    pub selected: bool,
}

/// The cursor in viewport coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub shape: CursorShape,
    /// False when the program hides the cursor, or the view is scrolled so far back
    /// that the cursor is below it.
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CursorShape {
    #[default]
    Block,
    Underline,
    Beam,
    HollowBlock,
}

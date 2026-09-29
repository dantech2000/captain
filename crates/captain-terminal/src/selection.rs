/// A cell in viewport coordinates, plus the half of the cell the pointer is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPoint {
    /// 0 is the top row of the view.
    pub row: usize,
    pub col: usize,
    pub side: Side,
}

impl GridPoint {
    pub fn new(row: usize, col: usize, side: Side) -> Self {
        Self { row, col, side }
    }
}

/// The left or right half of a cell. A selection that starts on the right half of a
/// cell leaves that cell out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    Left,
    Right,
}

/// What a drag selects.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SelectionKind {
    /// Character by character, from a single click.
    #[default]
    Simple,
    /// Whole words, from a double click.
    Word,
    /// Whole lines, from a triple click.
    Line,
}

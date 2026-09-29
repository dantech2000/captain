//! Splits a row of cells into the pieces the grid paints in one call.

use std::ops::Range;

use captain_terminal::Cell;

/// Consecutive cells whose `key` is the same `Some` value, as `(columns, key)`. Cells
/// with `None` are left out. The grid uses it for background and selection rectangles.
pub fn spans_by<K: PartialEq>(
    row: &[Cell],
    key: impl Fn(&Cell) -> Option<K>,
) -> Vec<(Range<usize>, K)> {
    let mut spans: Vec<(Range<usize>, K)> = Vec::new();
    for (col, cell) in row.iter().enumerate() {
        let Some(value) = key(cell) else { continue };
        match spans.last_mut() {
            Some((range, last)) if range.end == col && *last == value => range.end = col + 1,
            _ => spans.push((col..col + 1, value)),
        }
    }
    spans
}

/// The column ranges to shape as one line of text. A wide character gets a range of its
/// own, so it can take two cells; its spacer is skipped. Blank cells at the end of the
/// row draw nothing, so they are left out.
pub fn text_segments(row: &[Cell]) -> Vec<Range<usize>> {
    let end = row
        .iter()
        .rposition(|cell| cell.c != ' ' || cell.flags.underline || cell.flags.strikeout)
        .map_or(0, |last| last + 1);
    let mut segments: Vec<Range<usize>> = Vec::new();
    let mut open: Option<usize> = None;
    for (col, cell) in row[..end].iter().enumerate() {
        if cell.flags.wide || cell.flags.wide_spacer {
            if let Some(start) = open.take() {
                segments.push(start..col);
            }
            if cell.flags.wide {
                segments.push(col..col + 1);
            }
        } else if open.is_none() {
            open = Some(col);
        }
    }
    if let Some(start) = open {
        segments.push(start..end);
    }
    segments
}

#[cfg(test)]
mod tests;

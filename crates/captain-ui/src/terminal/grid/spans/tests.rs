use captain_terminal::{Cell, CellFlags, Color};

use super::{spans_by, text_segments};

fn row(text: &str) -> Vec<Cell> {
    text.chars()
        .map(|c| Cell {
            c,
            ..Cell::default()
        })
        .collect()
}

#[test]
fn equal_neighbors_merge_into_one_span() {
    let mut cells = row("abcdef");
    for cell in &mut cells[1..3] {
        cell.bg = Color::Indexed(1);
    }
    cells[3].bg = Color::Indexed(2);
    cells[5].bg = Color::Indexed(1);
    let spans = spans_by(&cells, |cell| {
        (cell.bg != Color::Background).then_some(cell.bg)
    });
    assert_eq!(
        spans,
        [
            (1..3, Color::Indexed(1)),
            (3..4, Color::Indexed(2)),
            (5..6, Color::Indexed(1)),
        ]
    );
}

#[test]
fn segments_skip_trailing_blanks() {
    assert_eq!(text_segments(&row("ls -la   ")), vec![(0..6)]);
    assert!(text_segments(&row("    ")).is_empty());
}

#[test]
fn wide_characters_get_their_own_segment() {
    let mut cells = row("a漢 b");
    cells[1].flags = CellFlags {
        wide: true,
        ..CellFlags::default()
    };
    cells[2].flags = CellFlags {
        wide_spacer: true,
        ..CellFlags::default()
    };
    assert_eq!(text_segments(&cells), [0..1, 1..2, 3..4]);
}

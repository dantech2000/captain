use super::AlacrittyEmulator;
use crate::{Color, CursorShape, Emulator, GridPoint, Key, KeyInput, Rgb, SelectionKind, Side};

fn term(bytes: &[u8]) -> AlacrittyEmulator {
    let mut term = AlacrittyEmulator::new(20, 4);
    term.feed(bytes);
    term
}

#[test]
fn plain_text_moves_the_cursor() {
    let screen = term(b"hello\r\nworld").snapshot();
    assert_eq!((screen.cols, screen.rows), (20, 4));
    assert_eq!(screen.row_text(0), "hello");
    assert_eq!(screen.row_text(1), "world");
    assert_eq!((screen.cursor.row, screen.cursor.col), (1, 5));
    assert!(screen.cursor.visible);
    assert_eq!(screen.cursor.shape, CursorShape::Block);
}

#[test]
fn sgr_sets_colors_and_flags() {
    let screen =
        term(b"\x1b[1;31mR\x1b[0m\x1b[38;2;1;2;3;48;5;200mT\x1b[7;4;3mI\x1b[0mN").snapshot();
    let row = &screen.lines[0];
    assert_eq!(row[0].fg, Color::Indexed(1));
    assert!(row[0].flags.bold);
    assert_eq!(row[1].fg, Color::Rgb(Rgb::new(1, 2, 3)));
    assert_eq!(row[1].bg, Color::Indexed(200));
    assert!(row[2].flags.inverse && row[2].flags.underline && row[2].flags.italic);
    assert_eq!(row[3].fg, Color::Foreground);
    assert_eq!(row[3].bg, Color::Background);
    assert!(!row[3].flags.bold);
}

#[test]
fn bright_colors_keep_their_index() {
    let screen = term(b"\x1b[92mG\x1b[2;34mB").snapshot();
    assert_eq!(screen.lines[0][0].fg, Color::Indexed(10));
    assert_eq!(screen.lines[0][1].fg, Color::Indexed(4));
    assert!(screen.lines[0][1].flags.dim);
}

#[test]
fn cursor_moves_and_hides() {
    let screen = term(b"\x1b[3;7H\x1b[6 q").snapshot();
    assert_eq!((screen.cursor.row, screen.cursor.col), (2, 6));
    assert_eq!(screen.cursor.shape, CursorShape::Beam);
    let hidden = term(b"\x1b[?25l").snapshot();
    assert!(!hidden.cursor.visible);
}

#[test]
fn resize_changes_the_grid() {
    let mut term = term(b"abc");
    term.resize(40, 10);
    assert_eq!(term.size(), (40, 10));
    let screen = term.snapshot();
    assert_eq!((screen.cols, screen.rows, screen.lines.len()), (40, 10, 10));
    assert_eq!(screen.lines[0].len(), 40);
    assert_eq!(screen.row_text(0), "abc");
}

#[test]
fn history_scrolls_back() {
    let mut term = term(b"1\r\n2\r\n3\r\n4\r\n5\r\n6");
    let live = term.snapshot();
    assert_eq!(live.history, 2);
    assert_eq!(live.row_text(0), "3");
    term.scroll(1);
    let back = term.snapshot();
    assert_eq!(
        (back.display_offset, back.row_text(0)),
        (1, "2".to_string())
    );
    // The cursor is on the last live row, which is now below the view.
    assert!(!back.cursor.visible);
    term.scroll(10);
    assert_eq!(term.snapshot().display_offset, 2);
    term.scroll_to_bottom();
    let live = term.snapshot();
    assert_eq!(live.display_offset, 0);
    assert!(live.cursor.visible);
    assert_eq!(live.cursor.row, 3);
}

#[test]
fn app_cursor_mode_changes_arrow_keys() {
    let mut term = term(b"");
    assert_eq!(term.encode_key(KeyInput::new(Key::Up)).unwrap(), b"\x1b[A");
    term.feed(b"\x1b[?1h\x1b[?2004h");
    assert!(term.modes().app_cursor && term.modes().bracketed_paste);
    assert_eq!(term.encode_key(KeyInput::new(Key::Up)).unwrap(), b"\x1bOA");
    assert_eq!(term.encode_paste("x"), b"\x1b[200~x\x1b[201~");
}

#[test]
fn title_and_replies_are_reported() {
    let mut term = term(b"\x1b]0;my shell\x07ab\x1b[6n");
    assert_eq!(term.title().as_deref(), Some("my shell"));
    assert_eq!(term.take_replies(), b"\x1b[1;3R");
    assert!(term.take_replies().is_empty());
}

#[test]
fn a_drag_selects_text() {
    let mut term = term(b"hello world\r\nsecond");
    term.select_start(GridPoint::new(0, 6, Side::Left), SelectionKind::Simple);
    term.select_update(GridPoint::new(0, 10, Side::Right));
    assert_eq!(term.selection_text().as_deref(), Some("world"));
    let screen = term.snapshot();
    assert!(screen.lines[0][6].flags.selected);
    assert!(!screen.lines[0][5].flags.selected);

    term.select_start(GridPoint::new(1, 2, Side::Left), SelectionKind::Line);
    assert_eq!(term.selection_text().as_deref(), Some("second\n"));
    term.select_clear();
    assert_eq!(term.selection_text(), None);
}

#[test]
fn wide_characters_take_two_cells() {
    let screen = term("漢x".as_bytes()).snapshot();
    assert!(screen.lines[0][0].flags.wide);
    assert!(screen.lines[0][1].flags.wide_spacer);
    assert_eq!(screen.lines[0][2].c, 'x');
    assert_eq!(screen.row_text(0), "漢x");
}

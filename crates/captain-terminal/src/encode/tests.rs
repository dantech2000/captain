use super::{encode_key, encode_paste};
use crate::{InputModes, Key, KeyInput, Modifiers};

fn plain(key: Key) -> KeyInput {
    KeyInput::new(key)
}

fn with(key: Key, mods: Modifiers) -> KeyInput {
    KeyInput::new(key).with_mods(mods)
}

#[test]
fn keys_send_their_bytes() {
    let normal = InputModes::default();
    let app_cursor = InputModes {
        app_cursor: true,
        ..InputModes::default()
    };
    let newline = InputModes {
        newline: true,
        ..InputModes::default()
    };
    let cases: [(KeyInput, InputModes, Option<&[u8]>); 31] = [
        // Arrows follow the cursor key mode.
        (plain(Key::Up), normal, Some(b"\x1b[A")),
        (plain(Key::Down), normal, Some(b"\x1b[B")),
        (plain(Key::Right), normal, Some(b"\x1b[C")),
        (plain(Key::Left), normal, Some(b"\x1b[D")),
        (plain(Key::Up), app_cursor, Some(b"\x1bOA")),
        (plain(Key::Home), app_cursor, Some(b"\x1bOH")),
        // Modified arrows use the xterm parameter, in either mode.
        (
            with(Key::Left, Modifiers::CTRL),
            app_cursor,
            Some(b"\x1b[1;5D"),
        ),
        (with(Key::Up, Modifiers::SHIFT), normal, Some(b"\x1b[1;2A")),
        // Control keys send C0 codes.
        (with(Key::Char('c'), Modifiers::CTRL), normal, Some(&[0x03])),
        (with(Key::Char('d'), Modifiers::CTRL), normal, Some(&[0x04])),
        (with(Key::Char('['), Modifiers::CTRL), normal, Some(&[0x1b])),
        (with(Key::Char('@'), Modifiers::CTRL), normal, Some(&[0x00])),
        (with(Key::Char('1'), Modifiers::CTRL), normal, None),
        (plain(Key::Enter), normal, Some(b"\r")),
        (plain(Key::Enter), newline, Some(b"\r\n")),
        (plain(Key::Backspace), normal, Some(&[0x7f])),
        (
            with(Key::Backspace, Modifiers::ALT),
            normal,
            Some(b"\x1b\x7f"),
        ),
        (plain(Key::Tab), normal, Some(b"\t")),
        (with(Key::Tab, Modifiers::SHIFT), normal, Some(b"\x1b[Z")),
        (plain(Key::Escape), normal, Some(&[0x1b])),
        // Characters send their typed text; Alt adds an escape.
        (plain(Key::Char('a')), normal, Some(b"a")),
        (
            with(Key::Char('a'), Modifiers::SHIFT).with_text("A"),
            normal,
            Some(b"A"),
        ),
        (with(Key::Char('b'), Modifiers::ALT), normal, Some(b"\x1bb")),
        (plain(Key::Char('é')), normal, Some("é".as_bytes())),
        // Function and editing keys.
        (plain(Key::F(1)), normal, Some(b"\x1bOP")),
        (plain(Key::F(5)), normal, Some(b"\x1b[15~")),
        (plain(Key::F(12)), normal, Some(b"\x1b[24~")),
        (plain(Key::F(13)), normal, None),
        (plain(Key::Delete), normal, Some(b"\x1b[3~")),
        (
            with(Key::Delete, Modifiers::CTRL),
            normal,
            Some(b"\x1b[3;5~"),
        ),
        (plain(Key::PageUp), normal, Some(b"\x1b[5~")),
    ];
    for (input, modes, bytes) in cases {
        assert_eq!(
            encode_key(&input, modes).as_deref(),
            bytes,
            "{input:?} in {modes:?}"
        );
    }
}

#[test]
fn paste_is_bracketed_only_when_the_program_asks() {
    assert_eq!(
        encode_paste("ls\nwho\r\n", InputModes::default()),
        b"ls\rwho\r"
    );
    let bracketed = InputModes {
        bracketed_paste: true,
        ..InputModes::default()
    };
    assert_eq!(
        encode_paste("echo\x1b[201~ hi\x03\n", bracketed),
        b"\x1b[200~echo[201~ hi\n\x1b[201~"
    );
}

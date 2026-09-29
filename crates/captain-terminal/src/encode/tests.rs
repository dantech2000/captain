use super::{encode_key, encode_paste};
use crate::{InputModes, Key, KeyInput, Modifiers};

fn key(key: Key) -> Vec<u8> {
    encode_key(&KeyInput::new(key), InputModes::default()).expect("bytes")
}

fn app_cursor() -> InputModes {
    InputModes {
        app_cursor: true,
        ..InputModes::default()
    }
}

#[test]
fn arrows_follow_the_cursor_key_mode() {
    assert_eq!(key(Key::Up), b"\x1b[A");
    assert_eq!(key(Key::Down), b"\x1b[B");
    assert_eq!(key(Key::Right), b"\x1b[C");
    assert_eq!(key(Key::Left), b"\x1b[D");

    let up = encode_key(&KeyInput::new(Key::Up), app_cursor());
    assert_eq!(up.as_deref(), Some(&b"\x1bOA"[..]));
    let home = encode_key(&KeyInput::new(Key::Home), app_cursor());
    assert_eq!(home.as_deref(), Some(&b"\x1bOH"[..]));
}

#[test]
fn modified_arrows_use_the_xterm_parameter() {
    let ctrl_left = KeyInput::new(Key::Left).with_mods(Modifiers::CTRL);
    assert_eq!(
        encode_key(&ctrl_left, app_cursor()).as_deref(),
        Some(&b"\x1b[1;5D"[..])
    );
    let shift_up = KeyInput::new(Key::Up).with_mods(Modifiers::SHIFT);
    assert_eq!(
        encode_key(&shift_up, InputModes::default()).as_deref(),
        Some(&b"\x1b[1;2A"[..])
    );
}

#[test]
fn control_keys_send_c0_codes() {
    let ctrl = |c| {
        encode_key(
            &KeyInput::new(Key::Char(c)).with_mods(Modifiers::CTRL),
            InputModes::default(),
        )
    };
    assert_eq!(ctrl('c').as_deref(), Some(&[0x03][..]));
    assert_eq!(ctrl('d').as_deref(), Some(&[0x04][..]));
    assert_eq!(ctrl('[').as_deref(), Some(&[0x1b][..]));
    assert_eq!(ctrl('@').as_deref(), Some(&[0x00][..]));
    assert_eq!(ctrl('1'), None);
}

#[test]
fn enter_backspace_tab_and_escape() {
    assert_eq!(key(Key::Enter), b"\r");
    assert_eq!(key(Key::Backspace), [0x7f]);
    assert_eq!(key(Key::Tab), b"\t");
    assert_eq!(key(Key::Escape), [0x1b]);

    let newline = InputModes {
        newline: true,
        ..InputModes::default()
    };
    let enter = encode_key(&KeyInput::new(Key::Enter), newline);
    assert_eq!(enter.as_deref(), Some(&b"\r\n"[..]));
    let shift_tab = KeyInput::new(Key::Tab).with_mods(Modifiers::SHIFT);
    assert_eq!(
        encode_key(&shift_tab, InputModes::default()).as_deref(),
        Some(&b"\x1b[Z"[..])
    );
    let alt_backspace = KeyInput::new(Key::Backspace).with_mods(Modifiers::ALT);
    assert_eq!(
        encode_key(&alt_backspace, InputModes::default()).as_deref(),
        Some(&b"\x1b\x7f"[..])
    );
}

#[test]
fn characters_send_their_text() {
    assert_eq!(key(Key::Char('a')), b"a");
    let shifted = KeyInput::new(Key::Char('a'))
        .with_mods(Modifiers::SHIFT)
        .with_text("A");
    assert_eq!(
        encode_key(&shifted, InputModes::default()).as_deref(),
        Some(&b"A"[..])
    );
    let alt_b = KeyInput::new(Key::Char('b')).with_mods(Modifiers::ALT);
    assert_eq!(
        encode_key(&alt_b, InputModes::default()).as_deref(),
        Some(&b"\x1bb"[..])
    );
    assert_eq!(key(Key::Char('é')), "é".as_bytes());
}

#[test]
fn function_and_editing_keys() {
    assert_eq!(key(Key::F(1)), b"\x1bOP");
    assert_eq!(key(Key::F(5)), b"\x1b[15~");
    assert_eq!(key(Key::F(12)), b"\x1b[24~");
    assert_eq!(key(Key::Delete), b"\x1b[3~");
    assert_eq!(key(Key::PageUp), b"\x1b[5~");
    let ctrl_delete = KeyInput::new(Key::Delete).with_mods(Modifiers::CTRL);
    assert_eq!(
        encode_key(&ctrl_delete, InputModes::default()).as_deref(),
        Some(&b"\x1b[3;5~"[..])
    );
    assert_eq!(
        encode_key(&KeyInput::new(Key::F(13)), InputModes::default()),
        None
    );
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

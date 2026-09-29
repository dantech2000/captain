//! Turns keys and pastes into the bytes an xterm-compatible program expects. The
//! encoding depends only on [`InputModes`], so every emulator backend can share it.

use crate::{InputModes, Key, KeyInput, Modifiers};

const ESC: u8 = 0x1b;

/// The bytes for one key press, or `None` if the key sends nothing.
pub fn encode_key(input: &KeyInput, modes: InputModes) -> Option<Vec<u8>> {
    let mods = input.mods;
    let bytes = match input.key {
        Key::Char(c) => return char_key(c, input.text.as_deref(), mods),
        Key::Enter if modes.newline => b"\r\n".to_vec(),
        Key::Enter => b"\r".to_vec(),
        Key::Tab if mods.shift => b"\x1b[Z".to_vec(),
        Key::Tab => b"\t".to_vec(),
        Key::Backspace if mods.ctrl => vec![0x08],
        Key::Backspace => vec![0x7f],
        Key::Escape => vec![ESC],
        Key::Up => cursor_key(b'A', mods, modes),
        Key::Down => cursor_key(b'B', mods, modes),
        Key::Right => cursor_key(b'C', mods, modes),
        Key::Left => cursor_key(b'D', mods, modes),
        Key::Home => cursor_key(b'H', mods, modes),
        Key::End => cursor_key(b'F', mods, modes),
        Key::Insert => tilde_key(2, mods),
        Key::Delete => tilde_key(3, mods),
        Key::PageUp => tilde_key(5, mods),
        Key::PageDown => tilde_key(6, mods),
        Key::F(n @ 1..=4) => cursor_like(b"PQRS"[usize::from(n - 1)], mods, true),
        Key::F(n @ 5..=12) => tilde_key([15, 17, 18, 19, 20, 21, 23, 24][usize::from(n - 5)], mods),
        Key::F(_) => return None,
    };
    // Enter, Tab, Backspace, and Escape take the Alt modifier as an ESC prefix.
    let plain_key = matches!(
        input.key,
        Key::Enter | Key::Tab | Key::Backspace | Key::Escape
    );
    if plain_key && mods.alt {
        Some([&[ESC][..], &bytes].concat())
    } else {
        Some(bytes)
    }
}

/// The bytes for a paste. Bracketed paste mode wraps the text, so the shell does not
/// run it line by line; ESC is removed so the text cannot end the bracket early.
/// Without it, line ends become CR, as if the user pressed Enter.
pub fn encode_paste(text: &str, modes: InputModes) -> Vec<u8> {
    if modes.bracketed_paste {
        let clean = text.replace('\x1b', "");
        [b"\x1b[200~", clean.as_bytes(), b"\x1b[201~"].concat()
    } else {
        text.replace("\r\n", "\r").replace('\n', "\r").into_bytes()
    }
}

/// A character key. Ctrl makes a control code, and Alt adds an ESC prefix.
fn char_key(c: char, text: Option<&str>, mods: Modifiers) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    if mods.alt {
        bytes.push(ESC);
    }
    if mods.ctrl {
        bytes.push(control_code(c)?);
    } else {
        match text {
            Some(text) if !text.is_empty() => bytes.extend_from_slice(text.as_bytes()),
            _ => {
                let mut buf = [0; 4];
                bytes.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            }
        }
    }
    Some(bytes)
}

/// The C0 code for ctrl plus `c`, as xterm sends it.
fn control_code(c: char) -> Option<u8> {
    let c = c.to_ascii_lowercase();
    match c {
        'a'..='z' => Some(c as u8 - b'a' + 1),
        '@' | ' ' | '2' => Some(0),
        '[' | '3' => Some(0x1b),
        '\\' | '4' => Some(0x1c),
        ']' | '5' => Some(0x1d),
        '^' | '6' => Some(0x1e),
        '_' | '-' | '7' => Some(0x1f),
        '?' | '8' => Some(0x7f),
        _ => None,
    }
}

/// Arrows, Home, and End: `ESC [ A`, or `ESC O A` in application cursor mode. With
/// modifiers, xterm always sends `ESC [ 1 ; m A`.
fn cursor_key(final_byte: u8, mods: Modifiers, modes: InputModes) -> Vec<u8> {
    cursor_like(final_byte, mods, modes.app_cursor)
}

fn cursor_like(final_byte: u8, mods: Modifiers, ss3: bool) -> Vec<u8> {
    if mods.any() {
        format!("\x1b[1;{}{}", mods.xterm_param(), final_byte as char).into_bytes()
    } else if ss3 {
        vec![ESC, b'O', final_byte]
    } else {
        vec![ESC, b'[', final_byte]
    }
}

/// Keys like Delete and Page Up: `ESC [ 3 ~`, or `ESC [ 3 ; m ~` with modifiers.
fn tilde_key(code: u8, mods: Modifiers) -> Vec<u8> {
    if mods.any() {
        format!("\x1b[{code};{}~", mods.xterm_param()).into_bytes()
    } else {
        format!("\x1b[{code}~").into_bytes()
    }
}

#[cfg(test)]
mod tests;

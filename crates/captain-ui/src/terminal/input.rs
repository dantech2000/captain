//! Maps GPUI keystrokes to terminal keys and to the copy and paste shortcuts.

use captain_terminal::{Key, KeyInput, Modifiers};
use gpui_kit::Keystroke;

/// A keystroke the terminal handles itself instead of sending it to the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shortcut {
    Copy,
    Paste,
}

/// Cmd-C and Cmd-V on macOS. Elsewhere Ctrl-C belongs to the program, so copy and
/// paste are Ctrl-Shift-C and Ctrl-Shift-V, as in most Linux terminals.
pub fn shortcut(keystroke: &Keystroke, macos: bool) -> Option<Shortcut> {
    let mods = &keystroke.modifiers;
    let chord = if macos {
        mods.platform && !mods.control && !mods.alt
    } else {
        mods.control && mods.shift && !mods.alt && !mods.platform
    };
    match keystroke.key.as_str() {
        "c" if chord => Some(Shortcut::Copy),
        "v" if chord => Some(Shortcut::Paste),
        _ => None,
    }
}

/// The terminal key for a keystroke, or `None` for keystrokes the terminal leaves to
/// the app, such as anything with Cmd. On macOS, Option types the character it
/// composes (Option-E then E types é) instead of sending an Escape prefix.
pub fn key_input(keystroke: &Keystroke, macos: bool) -> Option<KeyInput> {
    let mods = &keystroke.modifiers;
    if mods.platform {
        return None;
    }
    let mut input_mods = Modifiers {
        ctrl: mods.control,
        alt: mods.alt,
        shift: mods.shift,
    };
    let key = match keystroke.key.as_str() {
        "enter" => Key::Enter,
        "tab" => Key::Tab,
        "backspace" => Key::Backspace,
        "escape" => Key::Escape,
        "up" => Key::Up,
        "down" => Key::Down,
        "left" => Key::Left,
        "right" => Key::Right,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "insert" => Key::Insert,
        "delete" => Key::Delete,
        "space" => Key::Char(' '),
        name => match function_key(name) {
            Some(n) => Key::F(n),
            None => Key::Char(single_char(name)?),
        },
    };
    let mut input = KeyInput::new(key);
    if let Key::Char(c) = key {
        let typed = keystroke.key_char.clone().filter(|text| !text.is_empty());
        let composed = typed.as_deref().is_some_and(|text| text != c.to_string());
        if macos && input_mods.alt && !input_mods.ctrl && composed {
            input_mods.alt = false;
        }
        let text = typed
            .or_else(|| (input_mods.shift && !input_mods.ctrl).then(|| c.to_uppercase().collect()));
        if let Some(text) = text {
            input = input.with_text(text);
        }
    }
    Some(input.with_mods(input_mods))
}

fn function_key(name: &str) -> Option<u8> {
    let n: u8 = name.strip_prefix('f')?.parse().ok()?;
    (1..=12).contains(&n).then_some(n)
}

fn single_char(name: &str) -> Option<char> {
    let mut chars = name.chars();
    let c = chars.next()?;
    chars.next().is_none().then_some(c)
}

#[cfg(test)]
mod tests;

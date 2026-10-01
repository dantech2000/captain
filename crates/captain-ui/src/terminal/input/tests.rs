use captain_terminal::{Key, KeyInput, Modifiers as TermMods};
use gpui_kit::{Keystroke, Modifiers};

use super::{Shortcut, key_input, shortcut};

fn stroke(key: &str, key_char: Option<&str>, modifiers: Modifiers) -> Keystroke {
    Keystroke {
        modifiers,
        key: key.into(),
        key_char: key_char.map(str::to_string),
    }
}

fn plain(key: &str, key_char: Option<&str>) -> Keystroke {
    stroke(key, key_char, Modifiers::default())
}

#[test]
fn named_keys_map_to_terminal_keys() {
    assert_eq!(
        key_input(&plain("enter", None), true),
        Some(KeyInput::new(Key::Enter))
    );
    assert_eq!(
        key_input(&plain("up", None), false),
        Some(KeyInput::new(Key::Up))
    );
    assert_eq!(
        key_input(&plain("f5", None), false),
        Some(KeyInput::new(Key::F(5)))
    );
    assert_eq!(key_input(&plain("f20", None), false), None);
    assert_eq!(
        key_input(&plain("space", Some(" ")), false),
        Some(KeyInput::new(Key::Char(' ')).with_text(" "))
    );
}

#[test]
fn characters_carry_their_typed_text() {
    let shifted = stroke("a", Some("A"), Modifiers::shift());
    assert_eq!(
        key_input(&shifted, true),
        Some(
            KeyInput::new(Key::Char('a'))
                .with_text("A")
                .with_mods(TermMods::SHIFT)
        )
    );
    let ctrl_c = stroke("c", None, Modifiers::control());
    assert_eq!(
        key_input(&ctrl_c, true),
        Some(KeyInput::new(Key::Char('c')).with_mods(TermMods::CTRL))
    );
}

#[test]
fn option_composes_on_macos_and_is_meta_elsewhere() {
    let option_s = stroke("s", Some("ß"), Modifiers::alt());
    assert_eq!(
        key_input(&option_s, true),
        Some(KeyInput::new(Key::Char('s')).with_text("ß"))
    );
    assert_eq!(
        key_input(&stroke("b", Some("b"), Modifiers::alt()), false),
        Some(
            KeyInput::new(Key::Char('b'))
                .with_text("b")
                .with_mods(TermMods::ALT)
        )
    );
}

#[test]
fn cmd_keys_stay_with_the_app() {
    assert_eq!(
        key_input(&stroke("k", None, Modifiers::command()), true),
        None
    );
}

#[test]
fn copy_and_paste_follow_the_platform() {
    let cmd_c = stroke("c", None, Modifiers::command());
    assert_eq!(shortcut(&cmd_c, true), Some(Shortcut::Copy));
    let ctrl_c = stroke("c", None, Modifiers::control());
    assert_eq!(shortcut(&ctrl_c, false), None);
    let ctrl_shift_v = stroke(
        "v",
        None,
        Modifiers {
            control: true,
            shift: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(shortcut(&ctrl_shift_v, false), Some(Shortcut::Paste));
    assert_eq!(shortcut(&ctrl_shift_v, true), None);
}

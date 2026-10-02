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
fn keystrokes_map_to_terminal_keys() {
    let char_key = |c| KeyInput::new(Key::Char(c));
    let cases = [
        (plain("enter", None), true, Some(KeyInput::new(Key::Enter))),
        (plain("up", None), false, Some(KeyInput::new(Key::Up))),
        (plain("f5", None), false, Some(KeyInput::new(Key::F(5)))),
        (plain("f20", None), false, None),
        (
            plain("space", Some(" ")),
            false,
            Some(char_key(' ').with_text(" ")),
        ),
        // Characters carry their typed text.
        (
            stroke("a", Some("A"), Modifiers::shift()),
            true,
            Some(char_key('a').with_text("A").with_mods(TermMods::SHIFT)),
        ),
        (
            stroke("c", None, Modifiers::control()),
            true,
            Some(char_key('c').with_mods(TermMods::CTRL)),
        ),
        // Option composes on macOS and is Meta elsewhere.
        (
            stroke("s", Some("ß"), Modifiers::alt()),
            true,
            Some(char_key('s').with_text("ß")),
        ),
        (
            stroke("b", Some("b"), Modifiers::alt()),
            false,
            Some(char_key('b').with_text("b").with_mods(TermMods::ALT)),
        ),
        // Cmd keys stay with the app.
        (stroke("k", None, Modifiers::command()), true, None),
    ];
    for (keystroke, macos, input) in cases {
        assert_eq!(key_input(&keystroke, macos), input, "{keystroke:?}");
    }
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

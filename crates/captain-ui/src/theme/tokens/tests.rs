use captain_core::settings::ThemeFamily;

use super::Tokens;
use crate::theme::contrast;

#[test]
fn text_and_actions_meet_4_5_to_1_in_every_theme() {
    for family in ThemeFamily::ALL {
        for dark in [false, true] {
            let t = Tokens::of(family, dark);
            let pairs = [
                ("text on window", t.text, t.window),
                ("text on card", t.text, t.card),
                ("on action on action", t.on_action, t.action),
            ];
            for (what, fg, bg) in pairs {
                let ratio = contrast(fg, bg);
                assert!(ratio >= 4.5, "{family:?} dark={dark}: {what} is {ratio:.2}");
            }
        }
    }
}

#[test]
fn action_fg_reaches_3_to_1_on_cards_and_window() {
    for family in ThemeFamily::ALL {
        for dark in [false, true] {
            let t = Tokens::of(family, dark);
            for bg in [t.card, t.window] {
                let ratio = contrast(t.action_fg(), bg);
                assert!(ratio >= 3.0, "{family:?} dark={dark}: {ratio:.2}");
            }
        }
    }
}

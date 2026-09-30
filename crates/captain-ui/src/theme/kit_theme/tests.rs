use captain_core::settings::ThemeFamily;

use super::config;
use crate::theme::{Tokens, contrast};

#[test]
fn editor_colors_are_readable_in_every_theme() {
    for family in ThemeFamily::ALL {
        for dark in [false, true] {
            assert!(config(family, dark).highlight.is_some());
            let s = Tokens::of(family, dark).syntax();
            let pairs = [
                ("text", s.text, 4.5),
                ("keys", s.key, 4.5),
                ("strings", s.string, 4.5),
                ("numbers", s.number, 4.5),
                ("keywords", s.keyword, 4.5),
                ("punctuation", s.punctuation, 3.0),
                ("comments", s.comment, 3.0),
                ("line numbers", s.line_number, 3.0),
            ];
            for (what, fg, min) in pairs {
                let ratio = contrast(fg, s.background);
                assert!(ratio >= min, "{family:?} dark={dark}: {what} is {ratio:.2}");
            }
        }
    }
}

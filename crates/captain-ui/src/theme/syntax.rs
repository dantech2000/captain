//! Code editor colors, from the theme tokens: the Files tab highlights Compose
//! files and Dockerfiles with these.

use super::{Tokens, contrast};

/// The colors of the code editor in one theme and mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Syntax {
    /// The editor and its line-number gutter.
    pub background: u32,
    pub text: u32,
    /// Mapping keys and Dockerfile arguments.
    pub key: u32,
    pub string: u32,
    /// Numbers, booleans, and null.
    pub number: u32,
    /// Dockerfile instructions, YAML tags and anchors.
    pub keyword: u32,
    /// `:`, `-`, `,`, and brackets.
    pub punctuation: u32,
    pub comment: u32,
    pub line_number: u32,
    pub active_line_number: u32,
}

impl Tokens {
    pub fn syntax(&self) -> Syntax {
        let readable = |color| self.readable_on(color, self.card);
        Syntax {
            background: self.card,
            text: self.text,
            key: readable(self.link),
            string: readable(self.running),
            number: readable(self.warning_text),
            keyword: readable(self.info),
            punctuation: self.text3,
            comment: self.text3,
            line_number: self.text3,
            active_line_number: self.text,
        }
    }

    /// `color`, moved toward the text color in 10% steps until it reaches 4.5:1
    /// on `bg`. Periwinkle Light's green and blue need one step.
    fn readable_on(&self, color: u32, bg: u32) -> u32 {
        (0..=10)
            .map(|step| mix(color, self.text, step as f32 / 10.))
            .find(|&mixed| contrast(mixed, bg) >= 4.5)
            .unwrap_or(self.text)
    }
}

/// `a` moved `amount` (0 to 1) of the way to `b`, per channel.
fn mix(a: u32, b: u32, amount: f32) -> u32 {
    [16, 8, 0].into_iter().fold(0, |out, shift| {
        let (a, b) = (((a >> shift) & 0xff) as f32, ((b >> shift) & 0xff) as f32);
        out | (((a + (b - a) * amount).round() as u32) << shift)
    })
}

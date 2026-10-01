//! The terminal's colors: the theme's text and background, and 16 ANSI colors tuned
//! to Captain's dark and light palettes.

use captain_terminal::{Color, Rgb};
use gpui_kit::*;

use crate::theme::Palette;

const DARK_ANSI: [u32; 16] = [
    0x2c2c30, 0xff6961, 0x32d74b, 0xffd60a, 0x5aa9ff, 0xc38bff, 0x40c8e0, 0xd1d1d6, //
    0x6e6e76, 0xff8a82, 0x68e07c, 0xffe45e, 0x86c0ff, 0xd9aeff, 0x7dd9ea, 0xf5f5f7,
];

const LIGHT_ANSI: [u32; 16] = [
    0x1d1d1f, 0xd70015, 0x1f8a3a, 0x9a5b00, 0x0063d1, 0x9a2bc2, 0x0a7f96, 0xa1a1a8, //
    0x6e6e73, 0xff3b30, 0x28a745, 0xb86200, 0x007aff, 0xaf52de, 0x0891b2, 0x5f5f66,
];

/// The colors one frame of the terminal is drawn with.
#[derive(Debug, Clone, Copy)]
pub struct TerminalColors {
    pub foreground: Hsla,
    pub background: Hsla,
    pub cursor: Hsla,
    pub selection: Hsla,
    ansi: [Hsla; 16],
}

impl TerminalColors {
    pub fn new(palette: &Palette) -> Self {
        let table = if palette.dark { DARK_ANSI } else { LIGHT_ANSI };
        Self {
            foreground: palette.text,
            background: palette.terminal,
            cursor: palette.text,
            selection: palette.accent.alpha(if palette.dark { 0.38 } else { 0.24 }),
            ansi: table.map(|hex| rgb(hex).into()),
        }
    }

    /// The actual color of a cell color.
    pub fn resolve(&self, color: Color) -> Hsla {
        match color {
            Color::Foreground => self.foreground,
            Color::Background => self.background,
            Color::Indexed(index) => match Color::xterm_rgb(index) {
                Some(value) => hsla_of(value),
                None => self.ansi[usize::from(index)],
            },
            Color::Rgb(value) => hsla_of(value),
        }
    }
}

fn hsla_of(value: Rgb) -> Hsla {
    let hex = (u32::from(value.r) << 16) | (u32::from(value.g) << 8) | u32::from(value.b);
    rgb(hex).into()
}

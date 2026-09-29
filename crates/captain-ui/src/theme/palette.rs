//! Captain's colors for dark and light mode. See the v2 design canvas.

use captain_core::settings::Accent;
use gpui_kit::component::ActiveTheme;
use gpui_kit::*;

use super::accent_color;
use crate::settings;

/// Every color the Captain views use. Build it with [`Palette::of`].
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub bg: Hsla,
    pub side: Hsla,
    pub panel: Hsla,
    pub group: Hsla,
    pub card: Hsla,
    pub text: Hsla,
    pub text2: Hsla,
    pub text3: Hsla,
    pub sep: Hsla,
    pub field: Hsla,
    pub segment: Hsla,
    pub track: Hsla,
    pub button: Hsla,
    pub nav_selected: Hsla,
    pub terminal: Hsla,
    pub accent: Hsla,
    pub green: Hsla,
    pub orange: Hsla,
    pub red: Hsla,
    pub indigo: Hsla,
    pub teal: Hsla,
    pub gray: Hsla,
}

fn c(hex: u32) -> Hsla {
    rgb(hex).into()
}

fn ca(hex: u32) -> Hsla {
    rgba(hex).into()
}

impl Palette {
    /// The palette for the current appearance, with the accent from the settings.
    pub fn of(cx: &App) -> Self {
        let mut palette = if cx.theme().is_dark() {
            Self::dark_mode()
        } else {
            Self::light_mode()
        };
        palette.accent = accent_color(settings::accent(cx), palette.dark);
        palette
    }

    fn dark_mode() -> Self {
        Self {
            dark: true,
            bg: c(0x161618),
            side: c(0x1b1b1e),
            panel: c(0x19191c),
            group: ca(0xffffff06),
            card: ca(0xffffff09),
            text: c(0xf5f5f7),
            text2: c(0xa1a1a8),
            text3: c(0x6e6e76),
            sep: ca(0xffffff13),
            field: ca(0xffffff0f),
            segment: c(0x3a3a3f),
            track: ca(0xffffff14),
            button: ca(0xffffff0f),
            nav_selected: ca(0xffffff14),
            terminal: c(0x111113),
            accent: c(0x0a84ff),
            green: c(0x32d74b),
            orange: c(0xff9f0a),
            red: c(0xff6961),
            indigo: c(0x7d7aff),
            teal: c(0x40c8e0),
            gray: c(0x8e8e93),
        }
    }

    fn light_mode() -> Self {
        Self {
            dark: false,
            bg: c(0xffffff),
            side: c(0xf5f5f7),
            panel: c(0xfbfbfc),
            group: c(0xfbfbfc),
            card: c(0xffffff),
            text: c(0x1d1d1f),
            text2: c(0x5f5f66),
            text3: c(0x8e8e93),
            sep: ca(0x00000016),
            field: ca(0x0000000b),
            segment: c(0xffffff),
            track: ca(0x00000012),
            button: c(0xffffff),
            nav_selected: ca(0x0000000f),
            terminal: c(0xf6f6f8),
            accent: c(0x007aff),
            green: c(0x1f8a3a),
            orange: c(0xb86200),
            red: c(0xd70015),
            indigo: c(0x4f4cd6),
            teal: c(0x0a7f96),
            gray: c(0x6e6e73),
        }
    }

    /// A soft background version of `color`, for badges and icon tiles.
    pub fn tint(&self, color: Hsla) -> Hsla {
        color.alpha(if self.dark { 0.16 } else { 0.11 })
    }

    /// A stable color for a Compose project, picked from its name. It uses the default
    /// blue, not the accent, so a project keeps its color when the accent changes.
    pub fn project_color(&self, name: &str) -> Hsla {
        let blue = accent_color(Accent::Blue, self.dark);
        let choices = [self.indigo, self.teal, self.orange, self.green, blue];
        let sum: usize = name.bytes().map(usize::from).sum();
        choices[sum % choices.len()]
    }

    pub fn mono(&self) -> SharedString {
        if cfg!(target_os = "macos") {
            "Menlo".into()
        } else {
            "monospace".into()
        }
    }
}

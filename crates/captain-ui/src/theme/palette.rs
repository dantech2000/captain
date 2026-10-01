//! Captain's colors for the current theme and mode. See feature 0028.

use captain_core::settings::ThemeFamily;
use gpui_kit::component::ActiveTheme;
use gpui_kit::*;

use super::Tokens;
use crate::settings;

/// Every color the Captain views use. Build it with [`Palette::of`].
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub bg: Hsla,
    pub side: Hsla,
    /// The icon rail at the far left of the window.
    pub rail: Hsla,
    pub panel: Hsla,
    pub group: Hsla,
    pub card: Hsla,
    pub text: Hsla,
    pub text2: Hsla,
    pub text3: Hsla,
    pub sep: Hsla,
    pub border_strong: Hsla,
    pub field: Hsla,
    pub segment: Hsla,
    pub track: Hsla,
    pub button: Hsla,
    /// The background of a row under the mouse.
    pub hover: Hsla,
    pub nav_selected: Hsla,
    pub terminal: Hsla,
    /// The action color: buttons, focus, and selection. Never a state.
    pub accent: Hsla,
    /// Text on [`Palette::accent`].
    pub on_accent: Hsla,
    /// Links, and the action color when it colors text.
    pub link: Hsla,
    /// The action color for icons and thin lines: the link color where the action
    /// color is too faint on the background, as in Harbor light.
    pub accent_fg: Hsla,
    /// Running and healthy.
    pub green: Hsla,
    /// Warning, paused, and starting.
    pub orange: Hsla,
    /// [`Palette::orange`] for text.
    pub warn_text: Hsla,
    /// Failing and delete.
    pub red: Hsla,
    /// Text on [`Palette::red`].
    pub on_red: Hsla,
    /// Notes and neutral facts.
    pub info: Hsla,
    /// A label color, for charts and tiles that show no state.
    pub indigo: Hsla,
    /// The info color, under its old name.
    pub teal: Hsla,
    /// Stopped, created, and other quiet states.
    pub gray: Hsla,
    /// Colors that tell projects apart, and nothing else.
    pub labels: [Hsla; 5],
}

fn c(hex: u32) -> Hsla {
    rgb(hex).into()
}

impl Palette {
    /// The palette for the current appearance and the theme from the settings.
    pub fn of(cx: &App) -> Self {
        Self::new(settings::theme_family(cx), cx.theme().is_dark())
    }

    pub fn new(family: ThemeFamily, dark: bool) -> Self {
        let t = Tokens::of(family, dark);
        let text = c(t.text);
        let accent = c(t.action);
        let info = c(t.info);
        Self {
            dark,
            bg: c(t.window),
            side: c(t.sidebar),
            rail: c(t.rail),
            panel: c(t.sidebar),
            group: c(t.card),
            card: c(t.card),
            text,
            text2: c(t.text2),
            text3: c(t.text3),
            sep: c(t.border),
            border_strong: c(t.border_strong),
            field: c(t.field),
            segment: c(if dark { t.button } else { t.card }),
            track: c(t.border),
            button: c(t.button),
            hover: text.alpha(if dark { 0.06 } else { 0.05 }),
            nav_selected: accent.alpha(if dark { 0.22 } else { 0.14 }),
            terminal: c(t.log_panel),
            accent,
            on_accent: c(t.on_action),
            link: c(t.link),
            accent_fg: c(t.action_fg()),
            green: c(t.running),
            orange: c(t.warning),
            warn_text: c(t.warning_text),
            red: c(t.failing),
            on_red: c(t.on(t.failing)),
            info,
            indigo: c(t.labels[0]),
            teal: info,
            gray: c(t.text3),
            labels: t.labels.map(c),
        }
    }

    /// A soft background version of `color`, for badges and icon tiles.
    pub fn tint(&self, color: Hsla) -> Hsla {
        color.alpha(if self.dark { 0.16 } else { 0.11 })
    }

    /// The text version of a fill color: the link color for the action color, and the
    /// warning text color for the warning color. Other colors stay as they are.
    pub fn readable(&self, color: Hsla) -> Hsla {
        if color == self.accent {
            self.link
        } else if color == self.orange {
            self.warn_text
        } else {
            color
        }
    }

    /// A stable label color for a Compose project, picked from its name.
    pub fn project_color(&self, name: &str) -> Hsla {
        let sum: usize = name.bytes().map(usize::from).sum();
        self.labels[sum % self.labels.len()]
    }

    pub fn mono(&self) -> SharedString {
        if cfg!(target_os = "macos") {
            "Menlo".into()
        } else {
            "monospace".into()
        }
    }
}

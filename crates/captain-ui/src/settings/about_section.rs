use gpui_kit::*;

use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, settings_card, settings_row, text_button};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// The About card: version, license, and links to the project.
pub fn render(palette: &Palette) -> Div {
    let value = |text: &'static str| div().text_color(palette.text2).child(text);
    let link = |id: &'static str, label: &'static str, url: String| {
        text_button(
            id,
            label,
            ButtonTone::Accent,
            true,
            palette,
            move |_, _, cx| cx.open_url(&url),
        )
    };

    settings_card(
        "About",
        [
            settings_row("Captain", None, value(VERSION), palette).into_any_element(),
            settings_row("License", None, value(LICENSE), palette).into_any_element(),
            settings_row(
                "Source code",
                Some(REPOSITORY.trim_start_matches("https://").into()),
                link("about-source", "Open", REPOSITORY.to_string()),
                palette,
            )
            .id("settings-source")
            .help("Open Captain's source code in your browser.")
            .into_any_element(),
            settings_row(
                "Feedback",
                Some("Report a bug or ask for a feature.".into()),
                link(
                    "about-issues",
                    "Open issues",
                    format!("{REPOSITORY}/issues"),
                ),
                palette,
            )
            .id("settings-feedback")
            .help("Open Captain's issues in your browser, to report a bug or ask for a feature.")
            .into_any_element(),
        ],
        palette,
    )
}

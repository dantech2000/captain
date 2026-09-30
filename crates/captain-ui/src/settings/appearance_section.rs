use captain_core::settings::{Appearance, Settings, ThemeFamily};
use gpui_kit::*;

use super::store;
use super::theme_card::theme_card;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{Segment, segmented, settings_card, settings_row};

/// The Appearance card: the color theme, and light or dark mode.
pub fn render(settings: &Settings, palette: &Palette) -> Div {
    let modes = Appearance::ALL
        .into_iter()
        .map(|appearance| Segment {
            label: appearance.label().into(),
            selected: settings.appearance == appearance,
            help: match appearance {
                Appearance::System => "Follow the light or dark mode of the system.",
                Appearance::Light => "Always use the light colors.",
                Appearance::Dark => "Always use the dark colors.",
            }
            .into(),
            on_click: Box::new(move |_, cx| {
                store::update(cx, |settings| settings.appearance = appearance)
            }),
        })
        .collect();

    let themes = div().flex().items_center().gap(px(8.)).children(
        ThemeFamily::ALL
            .into_iter()
            .map(|family| theme_card(family, settings.theme == family, palette)),
    );

    settings_card(
        "Appearance",
        [
            settings_row(
                "Theme",
                Some("Each theme has a light and a dark version.".into()),
                themes,
                palette,
            )
            .id("settings-theme")
            .help("Choose Captain's colors.")
            .into_any_element(),
            settings_row(
                "Appearance",
                Some("System follows the light or dark mode of your computer.".into()),
                segmented("appearance-mode", modes, palette),
                palette,
            )
            .id("settings-appearance")
            .help("Choose light or dark, or follow the system.")
            .into_any_element(),
        ],
        palette,
    )
}

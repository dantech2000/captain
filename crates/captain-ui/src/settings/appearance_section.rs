use captain_core::settings::{Appearance, Settings, ThemeFamily};
use gpui_kit::*;

use super::store;
use super::theme_card::theme_card;
use crate::theme::Palette;
use crate::widgets::{Segment, segmented, settings_card, settings_row};

/// The Appearance card: the color theme, and light or dark mode.
pub fn render(settings: &Settings, palette: &Palette) -> Div {
    let modes = Appearance::ALL
        .into_iter()
        .map(|appearance| Segment {
            label: appearance.label().into(),
            selected: settings.appearance == appearance,
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
            .into_any_element(),
            settings_row(
                "Appearance",
                Some("System follows the light or dark mode of your computer.".into()),
                segmented("appearance-mode", modes, palette),
                palette,
            )
            .into_any_element(),
        ],
        palette,
    )
}

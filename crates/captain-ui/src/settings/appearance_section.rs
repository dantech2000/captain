use captain_core::settings::{Accent, Appearance, Settings};
use gpui_kit::*;

use super::store;
use crate::theme::{Palette, accent_color};
use crate::widgets::{Segment, segmented, settings_card, settings_row, swatch};

/// The Appearance card: light or dark mode, and the accent color.
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

    let swatches =
        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .children(Accent::ALL.into_iter().enumerate().map(|(ix, accent)| {
                swatch(
                    ("accent-swatch", ix),
                    accent_color(accent, palette.dark),
                    settings.accent == accent,
                    move |_, _, cx| store::update(cx, |settings| settings.accent = accent),
                )
            }));

    settings_card(
        "Appearance",
        [
            settings_row(
                "Appearance",
                Some("System follows the light or dark mode of your computer.".into()),
                segmented("appearance-mode", modes, palette),
                palette,
            )
            .into_any_element(),
            settings_row(
                "Accent color",
                Some(settings.accent.label().into()),
                swatches,
                palette,
            )
            .into_any_element(),
        ],
        palette,
    )
}

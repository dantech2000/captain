use captain_core::settings::{Appearance, Settings, ThemeFamily};
use gpui_kit::*;

use super::page_section::{row, section};
use super::store;
use super::theme_card::theme_card;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{Segment, segmented};

/// The Appearance section: the three themes, and System, Light, or Dark.
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

    let themes = div()
        .id("settings-theme")
        .flex_1()
        .flex()
        .items_center()
        .gap(px(8.))
        .children(
            ThemeFamily::ALL
                .into_iter()
                .map(|family| theme_card(family, settings.theme == family, palette)),
        )
        .help("Choose Captain's colors. Each theme has a light and a dark version.");

    section(palette).child(
        row("Appearance", palette).child(themes).child(
            div()
                .id("settings-appearance")
                .child(segmented("appearance-mode", modes, palette))
                .help("Choose light or dark, or follow the system."),
        ),
    )
}

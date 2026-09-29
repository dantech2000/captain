use captain_core::EngineError;
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::settings;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

/// Shown when Captain cannot reach its engine.
pub fn render(error: &EngineError, palette: &Palette) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .p(px(40.))
        .child(
            div()
                .w(px(520.))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.))
                .child(
                    div()
                        .size(px(64.))
                        .rounded(px(18.))
                        .bg(palette.tint(palette.red))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            Icon::new(IconName::Container)
                                .size(px(30.))
                                .text_color(palette.red),
                        ),
                )
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(FontWeight::BOLD)
                        .child("Captain's engine isn't running"),
                )
                .child(
                    div()
                        .text_color(palette.text2)
                        .child("Start the engine, then click Retry. Settings can switch engines."),
                )
                .child(text_button(
                    "engine-retry",
                    "Retry",
                    ButtonTone::Accent,
                    true,
                    palette,
                    |_, _, cx| settings::retry(cx),
                ))
                .child(
                    div()
                        .w_full()
                        .p(px(12.))
                        .rounded(px(10.))
                        .bg(palette.card)
                        .border_1()
                        .border_color(palette.sep)
                        .font_family(palette.mono())
                        .text_size(px(11.))
                        .text_color(palette.text2)
                        .child(error.to_string()),
                ),
        )
}

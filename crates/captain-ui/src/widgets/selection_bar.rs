use gpui_kit::*;

use super::{ButtonTone, text_button};
use crate::theme::Palette;

/// The bar above a list while several rows are selected: "n selected", the bulk
/// action buttons, and Clear.
pub fn selection_bar(
    count: usize,
    actions: Vec<Stateful<Div>>,
    palette: &Palette,
    on_clear: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
    div()
        .flex_shrink_0()
        .mx(px(24.))
        .mb(px(10.))
        .h(px(40.))
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(8.))
        .rounded(px(10.))
        .bg(palette.tint(palette.accent))
        .border_1()
        .border_color(palette.accent.alpha(0.35))
        .child(
            div()
                .flex_1()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.link)
                .child(format!("{count} selected")),
        )
        .children(actions)
        .child(text_button(
            "selection-clear",
            "Clear",
            ButtonTone::Accent,
            true,
            palette,
            on_clear,
        ))
}

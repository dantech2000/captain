use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::theme::Palette;

/// A tab that a later milestone fills in.
pub fn render(
    icon: IconName,
    title: &'static str,
    detail: &'static str,
    palette: &Palette,
) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(8.))
        .p(px(24.))
        .child(Icon::new(icon).size(px(28.)).text_color(palette.text3))
        .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
        .child(
            div()
                .text_size(px(12.))
                .text_color(palette.text2)
                .text_center()
                .child(detail),
        )
}

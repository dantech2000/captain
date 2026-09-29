use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::theme::Palette;

/// A centered icon, title, and hint for an empty list.
pub fn empty_note(
    icon: IconName,
    title: impl Into<SharedString>,
    hint: impl Into<SharedString>,
    palette: &Palette,
) -> Div {
    div()
        .pt(px(80.))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.))
        .text_color(palette.text2)
        .child(Icon::new(icon).size(px(32.)).text_color(palette.text3))
        .child(
            div()
                .text_color(palette.text)
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.into()),
        )
        .child(div().text_size(px(12.)).child(hint.into()))
}

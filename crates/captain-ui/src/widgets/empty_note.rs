use gpui_kit::*;

use crate::icons::{CaptainIcon, cap_icon};
use crate::theme::Palette;

/// A centered icon, title, and hint for an empty list.
pub fn empty_note(
    icon: CaptainIcon,
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
        .child(cap_icon(icon, px(32.), palette.text3))
        .child(
            div()
                .text_color(palette.text)
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.into()),
        )
        .child(div().text_size(px(12.)).child(hint.into()))
}

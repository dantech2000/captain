use gpui_kit::*;

use crate::theme::Palette;

/// A small outlined key label, such as `esc` or `⌘K`.
pub fn key_hint(text: impl Into<SharedString>, palette: &Palette) -> Div {
    div()
        .flex_shrink_0()
        .min_w(px(20.))
        .px(px(6.))
        .py(px(1.))
        .flex()
        .justify_center()
        .rounded(px(5.))
        .border_1()
        .border_color(palette.sep)
        .text_size(px(11.))
        .text_color(palette.text2)
        .child(text.into())
}

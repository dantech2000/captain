use gpui_kit::*;

/// A small rounded label, for states and health.
pub fn pill(text: impl Into<SharedString>, fg: Hsla, bg: Hsla) -> Div {
    div()
        .flex_shrink_0()
        .px(px(7.))
        .py(px(1.))
        .rounded(px(8.))
        .bg(bg)
        .text_color(fg)
        .text_size(px(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(text.into())
}

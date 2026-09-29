use gpui_kit::*;

use crate::theme::Palette;

/// One row of a [`super::settings_card`]: a label and an optional note on the left,
/// and a control or value on the right.
pub fn settings_row(
    label: impl IntoElement,
    note: Option<SharedString>,
    control: impl IntoElement,
    palette: &Palette,
) -> Div {
    div()
        .min_h(px(44.))
        .px(px(14.))
        .py(px(8.))
        .flex()
        .items_center()
        .gap(px(16.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(div().font_weight(FontWeight::MEDIUM).child(label))
                .children(note.map(|note| {
                    div()
                        .text_size(px(11.))
                        .text_color(palette.text2)
                        .child(note)
                })),
        )
        .child(
            div()
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(control),
        )
}

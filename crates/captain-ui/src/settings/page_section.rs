//! The building blocks of the one-page Settings: a section card with a label
//! column, and the small text beside a control. See feature 0037.

use gpui_kit::*;

use crate::theme::Palette;

/// The width of the label column. Notes under a row line up after it.
pub const LABEL_WIDTH: f32 = 150.;
/// The gap between the label column and the controls.
pub const GAP: f32 = 16.;

/// A section card with its rows in a column.
pub fn section(palette: &Palette) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .px(px(18.))
        .py(px(16.))
        .rounded(px(14.))
        .bg(palette.card)
        .border_1()
        .border_color(palette.sep)
}

/// A row: the label column, then `content`.
pub fn row(label: impl Into<SharedString>, palette: &Palette) -> Div {
    let label: SharedString = label.into();
    div().flex().items_center().gap(px(GAP)).child(
        div()
            .w(px(LABEL_WIDTH))
            .flex_shrink_0()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(palette.text)
            .child(label),
    )
}

/// A quiet label for a second row in a section, such as Resources.
pub fn sub_row(label: impl Into<SharedString>, palette: &Palette) -> Div {
    let label: SharedString = label.into();
    div().flex().items_center().gap(px(GAP)).child(
        div()
            .w(px(LABEL_WIDTH))
            .flex_shrink_0()
            .text_size(px(12.))
            .text_color(palette.text3)
            .child(label),
    )
}

/// A note under the controls, lined up with them.
pub fn under_note(text: impl Into<SharedString>, palette: &Palette) -> Div {
    let text: SharedString = text.into();
    div()
        .pl(px(LABEL_WIDTH + GAP))
        .text_size(px(11.5))
        .text_color(palette.text3)
        .child(text)
}

/// Quiet text that fills the rest of a row.
pub fn fill_note(text: impl Into<SharedString>, palette: &Palette) -> Div {
    let text: SharedString = text.into();
    div()
        .flex_1()
        .min_w_0()
        .text_size(px(12.5))
        .text_color(palette.text3)
        .child(text)
}

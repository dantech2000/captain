//! The rows of the New sheet's pickers, the template list and the image list,
//! spaced like the sheet's cards and the ⌘K palette's rows.

use gpui_kit::component::list::ListItem;
use gpui_kit::*;

use crate::theme::Palette;

/// A list item with room around its lines. The list draws the hover and the
/// selected background inside the rounded corners.
pub fn item(id: impl Into<ElementId>) -> ListItem {
    ListItem::new(id).px(px(10.)).py(px(10.)).rounded(px(8.))
}

/// A bold title over a smaller gray line.
pub fn lines(title: impl IntoElement, line: impl IntoElement, palette: &Palette) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(3.))
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .truncate()
                .child(title),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(palette.text2)
                .truncate()
                .child(line),
        )
}

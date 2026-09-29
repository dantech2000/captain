use gpui_kit::*;

use super::image_row::{CREATED_WIDTH, ID_WIDTH, SIZE_WIDTH};
use crate::theme::Palette;

/// Column names above the image card, lined up with the row cells.
pub fn render(palette: &Palette) -> impl IntoElement {
    let column = |label: &'static str, width: f32| div().w(px(width)).flex_shrink_0().child(label);
    div()
        .flex_shrink_0()
        .flex()
        .gap(px(12.))
        // The card padding (4) and border (1), plus the row padding (12), plus the list
        // padding (12), line these up with the row text.
        .pl(px(12. + 5. + 12.))
        .pr(px(12. + 5. + 12.))
        .py(px(8.))
        .border_t_1()
        .border_b_1()
        .border_color(palette.sep)
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(palette.text3)
        .child(div().flex_1().child("Repository"))
        .child(column("Image ID", ID_WIDTH))
        .child(column("Size", SIZE_WIDTH).text_right())
        .child(column("Created", CREATED_WIDTH).text_right())
}

use gpui_kit::*;

use crate::theme::Palette;

/// One fixed-width column after the name column. Right-aligned when `right` is set.
pub struct Column {
    pub label: &'static str,
    pub width: f32,
    pub right: bool,
}

/// Column names above list cards that use [`list_row`](super::list_row) with a
/// [`status_dot`](super::status_dot), lined up with the row cells.
pub fn column_header(columns: &[Column], palette: &Palette) -> Div {
    div()
        .flex_shrink_0()
        .flex()
        .gap(px(12.))
        // The card padding (4) and border (1), plus the row padding (12), plus the
        // state dot (22) and its gap (12), line these up with the row text.
        .pl(px(12. + 5. + 12. + 22. + 12.))
        .pr(px(12. + 5. + 12.))
        .py(px(8.))
        .border_t_1()
        .border_b_1()
        .border_color(palette.sep)
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(palette.text3)
        .child(div().flex_1().child("Name"))
        .children(columns.iter().map(|column| {
            let cell = div()
                .w(px(column.width))
                .flex_shrink_0()
                .child(column.label);
            if column.right {
                cell.text_right()
            } else {
                cell
            }
        }))
}

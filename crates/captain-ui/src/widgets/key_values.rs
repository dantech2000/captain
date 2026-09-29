use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;

/// One row of [`key_values`]: the key, the value, and the value's color.
pub type KeyValue = (String, String, Hsla);

/// A bordered list of key and value rows in monospace. Long values wrap.
pub fn colored_key_values(rows: Vec<KeyValue>, palette: &Palette) -> Div {
    div()
        .flex()
        .flex_col()
        .rounded(px(9.))
        .border_1()
        .border_color(palette.sep)
        .overflow_hidden()
        .font_family(palette.mono())
        .text_size(px(11.))
        .children(
            rows.into_iter()
                .enumerate()
                .map(|(ix, (key, value, color))| {
                    div()
                        .flex()
                        .gap(px(10.))
                        .px(px(10.))
                        .py(px(7.))
                        .when(ix > 0, |row| row.border_t_1().border_color(palette.sep))
                        .child(
                            div()
                                .w(px(120.))
                                .flex_shrink_0()
                                .truncate()
                                .text_color(palette.text2)
                                .child(key),
                        )
                        .child(div().flex_1().min_w_0().text_color(color).child(value))
                }),
        )
}

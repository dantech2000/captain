use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;

/// A small grey heading above a group of rows.
pub fn heading(text: &'static str, palette: &Palette) -> Div {
    div()
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(palette.text3)
        .child(text)
}

/// A bordered list of key and value rows in monospace.
pub fn key_values(rows: Vec<(String, String, Hsla)>, palette: &Palette) -> Div {
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
                                .w(px(130.))
                                .flex_shrink_0()
                                .truncate()
                                .text_color(palette.text2)
                                .child(key),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .text_color(color)
                                .child(value),
                        )
                }),
        )
}

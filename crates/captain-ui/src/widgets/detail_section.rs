use std::collections::BTreeMap;

use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;

/// A titled group in a detail panel: a small grey heading above `content`.
pub fn detail_section(
    title: impl Into<SharedString>,
    content: impl IntoElement,
    palette: &Palette,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text3)
                .child(title.into()),
        )
        .child(content)
}

/// Grey text for a section with nothing in it, such as "No labels".
pub fn detail_note(text: impl Into<SharedString>, palette: &Palette) -> Div {
    div()
        .text_size(px(12.))
        .text_color(palette.text3)
        .child(text.into())
}

/// A bordered list of key and value rows in monospace.
pub fn key_values(rows: Vec<(String, String)>, palette: &Palette) -> Div {
    div()
        .flex()
        .flex_col()
        .rounded(px(9.))
        .border_1()
        .border_color(palette.sep)
        .overflow_hidden()
        .font_family(palette.mono())
        .text_size(px(11.))
        .children(rows.into_iter().enumerate().map(|(ix, (key, value))| {
            div()
                .flex()
                .gap(px(10.))
                .px(px(10.))
                .py(px(7.))
                .when(ix > 0, |row| row.border_t_1().border_color(palette.sep))
                .child(
                    div()
                        .w(px(110.))
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
                        .text_color(palette.text)
                        .child(value),
                )
        }))
}

/// A map such as labels or driver options as [`key_values`], or `empty` when there
/// is nothing in it.
pub fn map_or_note(map: &BTreeMap<String, String>, empty: &'static str, palette: &Palette) -> Div {
    if map.is_empty() {
        return detail_note(empty, palette);
    }
    let rows = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    key_values(rows, palette)
}

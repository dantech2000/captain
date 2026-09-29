use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;

/// A titled group of settings rows, with a line between rows, like a macOS form.
pub fn settings_card(
    title: &'static str,
    rows: impl IntoIterator<Item = AnyElement>,
    palette: &Palette,
) -> Div {
    let sep = palette.sep;
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(
            div()
                .px(px(4.))
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text2)
                .child(title),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .rounded(px(12.))
                .bg(palette.group)
                .border_1()
                .border_color(palette.sep)
                .children(rows.into_iter().enumerate().map(move |(ix, row)| {
                    div()
                        .when(ix > 0, |this| this.border_t_1().border_color(sep))
                        .child(row)
                })),
        )
}

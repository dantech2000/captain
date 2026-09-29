use gpui_kit::*;

use super::{Scale, sparkline};
use crate::theme::Palette;

/// The content of one stat tile.
pub struct StatTile {
    pub label: &'static str,
    pub value: String,
    pub unit: String,
    pub color: Hsla,
    pub series: Vec<f64>,
    pub scale: Scale,
}

/// A card with a label, a large value, and a sparkline.
pub fn stat_tile(tile: StatTile, chart: Size<Pixels>, palette: &Palette) -> Div {
    let value_row = div()
        .flex()
        .items_end()
        .justify_between()
        .gap_2()
        .child(
            div()
                .flex()
                .items_baseline()
                .child(
                    div()
                        .text_size(px(20.))
                        .font_weight(FontWeight::BOLD)
                        .child(tile.value),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(palette.text3)
                        .child(tile.unit),
                ),
        )
        .child(
            sparkline(
                tile.series,
                tile.scale,
                tile.color,
                Some(palette.tint(tile.color)),
            )
            .w(chart.width)
            .h(chart.height),
        );

    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .px(px(14.))
        .py(px(12.))
        .rounded(px(12.))
        .bg(palette.card)
        .border_1()
        .border_color(palette.sep)
        .child(
            div()
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text2)
                .child(tile.label),
        )
        .child(value_row)
}

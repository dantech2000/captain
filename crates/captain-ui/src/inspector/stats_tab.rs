use captain_core::format::{bytes_label, percent_label, rate_label};
use captain_core::store::StatsHistory;
use gpui_kit::*;

use crate::theme::Palette;
use crate::widgets::{Scale, scales, sparkline};

/// Large CPU, memory, and network charts for the last 60 samples.
pub fn render(history: Option<&StatsHistory>, palette: &Palette) -> impl IntoElement {
    let Some(history) = history else {
        return div()
            .p(px(20.))
            .text_size(px(12.))
            .text_color(palette.text3)
            .child("Stats appear while the container runs.");
    };
    let latest = history.latest().copied().unwrap_or_default();
    let (rx, tx) = history.net_rate();
    let net: Vec<f64> = history
        .net_series()
        .iter()
        .map(|(rx, tx)| (rx + tx) as f64)
        .collect();

    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .px(px(20.))
        .pt(px(14.))
        .child(chart(
            "CPU",
            percent_label(latest.cpu_percent),
            history.cpu_series(),
            scales::CPU,
            palette.accent,
            palette,
        ))
        .child(chart(
            "Memory",
            bytes_label(latest.memory_bytes),
            history.memory_series(),
            scales::MEMORY,
            palette.indigo,
            palette,
        ))
        .child(chart(
            "Network I/O",
            rate_label(rx + tx),
            net,
            scales::NETWORK,
            palette.teal,
            palette,
        ))
}

fn chart(
    label: &'static str,
    value: String,
    series: Vec<f64>,
    scale: Scale,
    color: Hsla,
    palette: &Palette,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .px(px(14.))
        .py(px(12.))
        .rounded(px(12.))
        .bg(palette.card)
        .border_1()
        .border_color(palette.sep)
        .child(
            div()
                .flex()
                .items_baseline()
                .justify_between()
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(palette.text2)
                        .child(label),
                )
                .child(
                    div()
                        .text_size(px(18.))
                        .font_weight(FontWeight::BOLD)
                        .child(value),
                ),
        )
        .child(
            sparkline(series, scale, color, Some(palette.tint(color)))
                .w_full()
                .h(px(84.)),
        )
        .child(
            div()
                .flex()
                .justify_between()
                .text_size(px(10.))
                .text_color(palette.text3)
                .child("60s ago")
                .child("now"),
        )
}

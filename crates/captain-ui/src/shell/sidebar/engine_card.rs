use captain_core::format::{bytes_label, percent_label};
use gpui_kit::*;

use crate::theme::Palette;
use crate::workspace::{Connection, Workspace};

/// Engine resources and how much the containers use.
pub fn render(workspace: &Workspace, palette: &Palette) -> impl IntoElement {
    let Connection::Connected(info) = workspace.connection() else {
        return div();
    };
    let stats = workspace.stats();
    let cpu_capacity = f64::from(info.cpus.max(1)) * 100.0;
    let cpu = stats.total_cpu() / cpu_capacity;
    let memory = stats.total_memory() as f64 / info.memory_bytes.max(1) as f64;

    div()
        .flex()
        .flex_col()
        .gap(px(10.))
        .p(px(12.))
        .rounded(px(12.))
        .bg(palette.card)
        .border_1()
        .border_color(palette.sep)
        .child(
            div()
                .flex()
                .justify_between()
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Captain Engine"),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.text3)
                        .child(format!(
                            "{} CPUs · {}",
                            info.cpus,
                            bytes_label(info.memory_bytes)
                        )),
                ),
        )
        .child(gauge(
            "CPU",
            percent_label(cpu * 100.0),
            cpu,
            palette.accent,
            palette,
        ))
        .child(gauge(
            "Memory",
            format!(
                "{} of {}",
                bytes_label(stats.total_memory()),
                bytes_label(info.memory_bytes)
            ),
            memory,
            palette.accent,
            palette,
        ))
}

fn gauge(label: &'static str, value: String, fraction: f64, color: Hsla, palette: &Palette) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(5.))
        .child(
            div()
                .flex()
                .justify_between()
                .text_size(px(11.))
                .child(div().text_color(palette.text2).child(label))
                .child(value),
        )
        .child(
            div()
                .h(px(5.))
                .rounded(px(3.))
                .bg(palette.track)
                .overflow_hidden()
                .child(
                    div()
                        .h_full()
                        .rounded(px(3.))
                        .bg(color)
                        .w(relative(fraction.clamp(0.0, 1.0) as f32)),
                ),
        )
}

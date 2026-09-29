use captain_core::format::{bytes_label, percent_label, rate_label};
use gpui_kit::*;

use crate::theme::Palette;
use crate::widgets::{StatTile, scales, stat_tile};
use crate::workspace::Workspace;

/// Running count, and total CPU, memory, and network over all containers.
pub fn render(workspace: &Workspace, palette: &Palette) -> impl IntoElement {
    let (running, total) = workspace.shown_counts();
    let stats = workspace.stats();
    let (memory, memory_unit) = split_unit(bytes_label(stats.total_memory()));
    let (net, net_unit) = split_unit(rate_label(stats.total_net_rate()));
    let tiles = [
        StatTile {
            label: "Running",
            value: running.to_string(),
            unit: format!(" / {total}"),
            color: palette.green,
            series: Vec::new(),
            scale: scales::COUNT,
        },
        StatTile {
            label: "CPU",
            value: percent_label(stats.total_cpu())
                .trim_end_matches('%')
                .to_string(),
            unit: "%".into(),
            color: palette.accent,
            series: stats.total_cpu_series(),
            scale: scales::CPU,
        },
        StatTile {
            label: "Memory",
            value: memory,
            unit: memory_unit,
            color: palette.indigo,
            series: stats.total_memory_series(),
            scale: scales::MEMORY,
        },
        StatTile {
            label: "Network",
            value: net,
            unit: net_unit,
            color: palette.teal,
            series: stats.total_net_series(),
            scale: scales::NETWORK,
        },
    ];

    div()
        .flex_shrink_0()
        .flex()
        .gap(px(12.))
        .px(px(24.))
        .pb(px(16.))
        .children(tiles.into_iter().map(|tile| {
            stat_tile(tile, size(px(72.), px(26.)), palette)
                .flex_1()
                .min_w_0()
        }))
}

/// Splits `182 MB` into `182` and ` MB`.
fn split_unit(label: String) -> (String, String) {
    match label.split_once(' ') {
        Some((value, unit)) => (value.to_string(), format!(" {unit}")),
        None => (label, String::new()),
    }
}

use captain_core::format::{bytes_label, percent_label, rate_label};
use captain_core::model::{Container, ContainerDetail};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::section::{heading, key_values};
use crate::theme::Palette;
use crate::widgets::{StatTile, scales, stat_tile};
use crate::workspace::Workspace;

/// Live stats, ports, health, details, environment, and mounts.
pub fn render(
    container: &Container,
    detail: Option<&ContainerDetail>,
    workspace: &Workspace,
    palette: &Palette,
) -> impl IntoElement {
    let body = div()
        .flex()
        .flex_col()
        .gap(px(18.))
        .child(stats_grid(container, workspace, palette))
        .child(ports(container, palette));
    let body = match detail {
        Some(detail) => body
            .child(health(detail, palette))
            .child(details(container, detail, palette))
            .child(environment(detail, palette))
            .child(mounts(detail, palette)),
        None => body.child(
            div()
                .text_size(px(12.))
                .text_color(palette.text3)
                .child("Loading details..."),
        ),
    };

    div()
        .id("overview")
        .flex_1()
        .overflow_y_scroll()
        .px(px(20.))
        .pt(px(16.))
        .pb(px(20.))
        .child(body)
}

fn stats_grid(container: &Container, workspace: &Workspace, palette: &Palette) -> Div {
    let history = workspace.stats().get(&container.id);
    let latest = history.and_then(|h| h.latest()).copied();
    let (rx, tx) = history.map(|h| h.net_rate()).unwrap_or_default();
    let net = |pick: fn(&(u64, u64)) -> u64| {
        history
            .map(|h| {
                h.net_series()
                    .iter()
                    .map(|pair| pick(pair) as f64)
                    .collect()
            })
            .unwrap_or_default()
    };
    let dash = || "—".to_string();
    let tiles = [
        StatTile {
            label: "CPU",
            value: latest
                .map(|s| percent_label(s.cpu_percent))
                .unwrap_or_else(dash),
            unit: String::new(),
            color: palette.accent,
            series: history.map(|h| h.cpu_series()).unwrap_or_default(),
            scale: scales::CPU,
        },
        StatTile {
            label: "Memory",
            value: latest
                .map(|s| bytes_label(s.memory_bytes))
                .unwrap_or_else(dash),
            unit: String::new(),
            color: palette.indigo,
            series: history.map(|h| h.memory_series()).unwrap_or_default(),
            scale: scales::MEMORY,
        },
        StatTile {
            label: "Net in",
            value: latest.map(|_| rate_label(rx)).unwrap_or_else(dash),
            unit: String::new(),
            color: palette.teal,
            series: net(|pair| pair.0),
            scale: scales::NETWORK,
        },
        StatTile {
            label: "Net out",
            value: latest.map(|_| rate_label(tx)).unwrap_or_else(dash),
            unit: String::new(),
            color: palette.orange,
            series: net(|pair| pair.1),
            scale: scales::NETWORK,
        },
    ];
    let mut tiles = tiles.into_iter().map(|tile| {
        stat_tile(tile, size(px(60.), px(24.)), palette)
            .flex_1()
            .min_w_0()
    });
    let mut row = || div().flex().gap(px(10.)).children(tiles.by_ref().take(2));
    div()
        .flex()
        .flex_col()
        .gap(px(10.))
        .child(row())
        .child(row())
}

fn ports(container: &Container, palette: &Palette) -> Div {
    let rows: Vec<(String, String, Hsla)> = container
        .ports
        .iter()
        .map(|port| {
            let host = match port.public_port {
                Some(public) => format!("localhost:{public}"),
                None => "not published".to_string(),
            };
            (
                host,
                format!("{}/{}", port.private_port, port.protocol),
                palette.text,
            )
        })
        .collect();
    let list = if rows.is_empty() {
        div()
            .text_size(px(12.))
            .text_color(palette.text3)
            .child("No ports")
    } else {
        key_values(rows, palette)
    };
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(heading("Ports", palette))
        .child(list)
}

fn health(detail: &ContainerDetail, palette: &Palette) -> Div {
    let checks = &detail.health_checks;
    let note = if checks.is_empty() {
        "No health check".to_string()
    } else {
        let passed = checks.iter().filter(|c| c.passed).count();
        format!("{passed} of the last {} passed", checks.len())
    };
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .flex()
                .justify_between()
                .child(heading("Health checks", palette))
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.text2)
                        .child(note),
                ),
        )
        .when(!checks.is_empty(), |this| {
            this.child(
                div()
                    .flex()
                    .gap(px(3.))
                    .h(px(22.))
                    .children(checks.iter().map(|check| {
                        div().flex_1().rounded(px(2.)).bg(if check.passed {
                            palette.green
                        } else {
                            palette.red
                        })
                    })),
            )
        })
}

fn details(container: &Container, detail: &ContainerDetail, palette: &Palette) -> Div {
    let restart = if detail.restart_policy.is_empty() {
        "no".to_string()
    } else {
        detail.restart_policy.clone()
    };
    let rows = vec![
        (
            "Container ID".to_string(),
            container.short_id().to_string(),
            palette.text,
        ),
        ("Command".to_string(), detail.command.clone(), palette.text),
        (
            "Networks".to_string(),
            detail.networks.join(", "),
            palette.text,
        ),
        ("Restart policy".to_string(), restart, palette.text),
    ];
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(heading("Details", palette))
        .child(key_values(rows, palette))
}

fn environment(detail: &ContainerDetail, palette: &Palette) -> Div {
    let rows: Vec<_> = detail
        .env
        .iter()
        .map(|var| {
            let color = if var.is_secret() {
                palette.text3
            } else {
                palette.text
            };
            (var.key.clone(), var.display_value(), color)
        })
        .collect();
    let list = if rows.is_empty() {
        div()
            .text_size(px(12.))
            .text_color(palette.text3)
            .child("No variables")
    } else {
        key_values(rows, palette)
    };
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(heading("Environment", palette))
        .child(list)
}

fn mounts(detail: &ContainerDetail, palette: &Palette) -> Div {
    let rows: Vec<_> = detail
        .mounts
        .iter()
        .map(|mount| {
            (
                mount.source.clone(),
                mount.destination.clone(),
                palette.text2,
            )
        })
        .collect();
    let list = if rows.is_empty() {
        div()
            .text_size(px(12.))
            .text_color(palette.text3)
            .child("No mounts")
    } else {
        key_values(rows, palette)
    };
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(heading("Mounts", palette))
        .child(list)
}

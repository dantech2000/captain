use std::sync::Arc;

use captain_core::format::{bytes_label, percent_label, short_bytes_label, short_percent_label};
use captain_core::model::EngineInfo;
use gpui_kit::*;

use super::disk_summary::DiskSummary;
use super::power_button;
use crate::engine_host::HostSummary;
use crate::help::HelpExt;
use crate::shell::engine_state::EngineState;
use crate::theme::Palette;
use crate::workspace::{Connection, Page, Workspace};

/// One line under the projects: the engine's state dot, then CPU, memory, and disk
/// while connected, else the state; with Captain Engine a power button on the right.
/// Each part has its own help sentence. See feature 0030.
pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    host: Option<&HostSummary>,
    disk: Option<&DiskSummary>,
    palette: &Palette,
) -> Div {
    let engine = EngineState::of(workspace, host, palette);
    let mut parts: Vec<AnyElement> = Vec::new();
    match workspace.connection() {
        Connection::Connected(info) => {
            let [cpu, memory] = usage(workspace, info, palette);
            parts.push(cpu.into_any_element());
            parts.push(memory.into_any_element());
        }
        _ => parts.push(
            div()
                .id("sidebar-status-state")
                .text_color(palette.text2)
                .child(engine.state)
                .help(engine.help())
                .into_any_element(),
        ),
    }
    parts.extend(disk.map(|disk| disk_part(handle, disk, palette).into_any_element()));
    let mut row = div()
        .flex_1()
        .min_w_0()
        .flex()
        .items_center()
        .gap(px(6.))
        .overflow_hidden()
        .child(
            div()
                .id("sidebar-status-dot")
                .flex_shrink_0()
                .size(px(12.))
                .flex()
                .items_center()
                .justify_center()
                .child(div().size(px(7.)).rounded_full().bg(engine.color))
                .help(engine.help()),
        );
    for (index, part) in parts.into_iter().enumerate() {
        if index > 0 {
            row = row.child(div().text_color(palette.text3).child("·"));
        }
        row = row.child(part);
    }
    div()
        .flex_shrink_0()
        .h(px(34.))
        .mt(px(8.))
        .px(px(4.))
        .flex()
        .items_center()
        .gap(px(6.))
        .border_t_1()
        .border_color(palette.sep)
        .text_size(px(11.))
        .font_features(FontFeatures(Arc::new(vec![("tnum".into(), 1)])))
        .child(row)
        .children(host.map(|host| power_button::render(host, palette)))
}

/// CPU and memory of the containers, short, with the full numbers in the help.
fn usage(workspace: &Workspace, info: &EngineInfo, palette: &Palette) -> [Stateful<Div>; 2] {
    let stats = workspace.stats();
    let cpu = stats.total_cpu() / f64::from(info.cpus.max(1));
    let cpus = match info.cpus {
        1 => "1 CPU".to_string(),
        n => format!("{n} CPUs"),
    };
    [
        div()
            .id("sidebar-status-cpu")
            .text_color(palette.text2)
            .child(short_percent_label(cpu))
            .help(format!("CPU: {} of {cpus}.", percent_label(cpu))),
        div()
            .id("sidebar-status-memory")
            .text_color(palette.text2)
            .child(short_bytes_label(stats.total_memory()))
            .help(format!(
                "Memory: {} of {}.",
                bytes_label(stats.total_memory()),
                bytes_label(info.memory_bytes)
            )),
    ]
}

/// The disk in use, in the warning text color when a cleanup can free some. A click
/// opens Storage. See feature 0031.
fn disk_part(handle: &Entity<Workspace>, disk: &DiskSummary, palette: &Palette) -> Stateful<Div> {
    let handle = handle.clone();
    let breakdown = &disk.breakdown;
    let used = match breakdown.capacity {
        Some(capacity) => format!(
            "{} of {}",
            bytes_label(breakdown.used),
            bytes_label(capacity)
        ),
        None => format!("{} used", bytes_label(breakdown.used)),
    };
    let (color, help) = if disk.freeable > 0 {
        (
            palette.warn_text,
            format!(
                "Disk: {used}. {} can be freed. Click to review.",
                bytes_label(disk.freeable)
            ),
        )
    } else {
        (
            palette.text2,
            format!("Disk: {used}. Click to open Storage."),
        )
    };
    let hover = palette.text;
    div()
        .id("sidebar-status-disk")
        .text_color(color)
        .cursor_pointer()
        .hover(move |style| style.text_color(hover))
        .on_click(move |_, _, cx| {
            handle.update(cx, |workspace, cx| workspace.set_page(Page::Storage, cx))
        })
        .child(short_bytes_label(breakdown.used))
        .help(help)
}

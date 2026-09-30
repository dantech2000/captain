use captain_core::HostStatus;
use captain_core::format::{bytes_label, percent_label};
use captain_core::model::EngineInfo;
use gpui_kit::*;

use crate::engine_host::HostSummary;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};
use crate::workspace::{Connection, Workspace};

/// The engine's controls under the resource icons: CPU and memory gauges, and with
/// Captain Engine a Start or Stop button. The engine header at the top names the
/// engine and its state, so the card has no title. Empty gauges while the engine is
/// not connected keep its height, so the button does not move when it starts.
pub fn render(
    workspace: &Workspace,
    host: Option<&HostSummary>,
    palette: &Palette,
) -> impl IntoElement {
    let info = match workspace.connection() {
        Connection::Connected(info) => Some(info),
        _ => None,
    };
    if host.is_none() && info.is_none() {
        return div();
    }
    let gauges = match info {
        Some(info) => gauges(workspace, info, palette),
        None => [
            gauge("CPU", "—".into(), 0.0, palette.accent, palette),
            gauge("Memory", "—".into(), 0.0, palette.accent, palette),
        ],
    };
    div()
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(12.))
        .p(px(12.))
        .rounded(px(12.))
        .bg(palette.card)
        .border_1()
        .border_color(palette.sep)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(8.))
                .children(gauges),
        )
        .children(host.map(|host| control(host, palette)))
}

/// Start, Set up, or Stop.
fn control(host: &HostSummary, palette: &Palette) -> Stateful<Div> {
    let model = host.model.clone();
    if host.status.can_stop() {
        return text_button(
            "sidebar-engine-stop",
            "Stop",
            ButtonTone::Accent,
            host.can_control,
            palette,
            move |_, _, cx| model.update(cx, |model, cx| model.stop(cx)).detach(),
        )
        .help("Stop Captain Engine. Running containers stop with it.");
    }
    let label = if host.status == HostStatus::NotCreated {
        "Set up"
    } else {
        "Start"
    };
    text_button(
        "sidebar-engine-start",
        label,
        ButtonTone::Accent,
        host.can_control && host.status.can_start(),
        palette,
        move |_, _, cx| model.update(cx, |model, cx| model.start(cx)),
    )
    .help(if host.status == HostStatus::NotCreated {
        "Download and create the Captain Engine virtual machine."
    } else {
        "Start Captain Engine."
    })
}

/// CPU and memory use of the containers against the engine's capacity.
fn gauges(workspace: &Workspace, info: &EngineInfo, palette: &Palette) -> [Div; 2] {
    let stats = workspace.stats();
    let cpu_capacity = f64::from(info.cpus.max(1)) * 100.0;
    let cpu = stats.total_cpu() / cpu_capacity;
    let memory = stats.total_memory() as f64 / info.memory_bytes.max(1) as f64;
    [
        gauge(
            "CPU",
            percent_label(cpu * 100.0),
            cpu,
            palette.accent,
            palette,
        ),
        gauge(
            "Memory",
            format!(
                "{} of {}",
                bytes_label(stats.total_memory()),
                bytes_label(info.memory_bytes)
            ),
            memory,
            palette.accent,
            palette,
        ),
    ]
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

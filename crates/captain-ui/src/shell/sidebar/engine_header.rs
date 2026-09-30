use captain_core::format::bytes_label;
use gpui_kit::*;

use crate::engine_host::HostSummary;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::brand_mark;
use crate::workspace::{Connection, Workspace};

/// The app icon, the app name, and one line of engine state: for example `Running · 5 CPUs
/// · 463 MB`. With Captain Engine the state comes from the host, except that a
/// running engine that does not answer shows red. Its height never changes.
pub fn render(
    workspace: &Workspace,
    host: Option<&HostSummary>,
    palette: &Palette,
) -> Stateful<Div> {
    let connection = workspace.connection();
    let (color, state) = match (host, connection) {
        (Some(host), Connection::Failed(_)) if host.status.is_running() => {
            (palette.red, "Not answering")
        }
        (Some(host), _) => (palette.host_status(&host.status), host.status.label()),
        (None, Connection::Connecting) => (palette.orange, "Connecting"),
        (None, Connection::Connected(_)) => (palette.green, "Running"),
        (None, Connection::Failed(_)) => (palette.red, "Not answering"),
    };
    let line = match connection {
        Connection::Connected(info) => format!(
            "{state} · {} CPUs · {}",
            info.cpus,
            bytes_label(workspace.stats().total_memory())
        ),
        _ => state.to_string(),
    };
    let engine = if host.is_some() {
        "Captain Engine"
    } else {
        "The engine"
    };
    div()
        .id("sidebar-engine-header")
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(10.))
        .px(px(4.))
        .pb(px(12.))
        .child(brand_mark(px(36.)))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(FontWeight::BOLD)
                        .child("Captain"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(px(11.))
                        .text_color(palette.text2)
                        .child(div().size(px(6.)).rounded_full().bg(color))
                        .child(div().truncate().child(line)),
                ),
        )
        .help(format!(
            "{engine}: {}. Memory counts every container.",
            state.to_lowercase()
        ))
}

use gpui_kit::*;

use crate::engine_host::HostSummary;
use crate::help::HelpExt;
use crate::shell::status_bar::engine_name;
use crate::theme::Palette;
use crate::widgets::brand_mark;
use crate::workspace::{Connection, Workspace};

/// The app icon, the app name, and one line of engine state: for example `Captain
/// Engine · Running`. With Captain Engine the state comes from the host, except that a
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
    // The numbers live in the engine card and the status bar; here only which
    // engine it is and its state.
    let engine = match (host, connection) {
        (Some(_), _) => "Captain Engine",
        (None, Connection::Connected(info)) => engine_name(&info.endpoint),
        (None, _) => "Engine",
    };
    let line = format!("{engine} · {state}");
    div()
        .id("sidebar-engine-header")
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(8.))
        .pb(px(12.))
        // The artwork has the macOS icon margin built in; pull it left so the
        // squircle lines up with the search field below.
        .child(div().ml(px(-4.)).child(brand_mark(px(40.))))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(FontWeight::SEMIBOLD)
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
            "{engine}: {}. The engine card below shows its CPU and memory.",
            state.to_lowercase()
        ))
}

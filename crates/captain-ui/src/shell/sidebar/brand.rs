use gpui_kit::*;

use crate::engine_host::HostSummary;
use crate::theme::Palette;
use crate::widgets::brand_mark;
use crate::workspace::Connection;

/// The Captain mark, name, and engine state. With Captain Engine the state comes
/// from the host, except that a running engine that does not answer shows red.
pub fn render(
    connection: &Connection,
    host: Option<&HostSummary>,
    palette: &Palette,
) -> impl IntoElement {
    let (color, text) = match (host, connection) {
        (Some(host), Connection::Failed(_)) if host.status.is_running() => {
            (palette.red, "Engine not answering")
        }
        (Some(host), _) => host.brand_line(palette),
        (None, Connection::Connecting) => (palette.orange, "Connecting..."),
        (None, Connection::Connected(_)) => (palette.green, "Engine running"),
        (None, Connection::Failed(_)) => (palette.red, "Engine stopped"),
    };

    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .px(px(2.))
        .pt(px(4.))
        .pb(px(14.))
        .child(brand_mark(px(44.)))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(1.))
                .child(
                    div()
                        .text_size(px(15.))
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
                        .child(text),
                ),
        )
}

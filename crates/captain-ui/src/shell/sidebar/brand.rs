use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::theme::Palette;
use crate::workspace::Connection;

/// The Captain mark, name, and engine state.
pub fn render(connection: &Connection, palette: &Palette) -> impl IntoElement {
    let (color, text) = match connection {
        Connection::Connecting => (palette.orange, "Connecting..."),
        Connection::Connected(_) => (palette.green, "Engine running"),
        Connection::Failed(_) => (palette.red, "Engine stopped"),
    };

    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .px(px(6.))
        .pt(px(4.))
        .pb(px(14.))
        .child(
            div()
                .size(px(34.))
                .rounded(px(10.))
                .bg(palette.accent)
                .flex()
                .items_center()
                .justify_center()
                .shadow(vec![BoxShadow {
                    color: palette.accent.alpha(0.35),
                    offset: point(px(0.), px(4.)),
                    blur_radius: px(14.),
                    spread_radius: px(0.),
                    inset: false,
                }])
                .child(
                    Icon::new(IconName::ShipWheel)
                        .size(px(20.))
                        .text_color(gpui_kit::white()),
                ),
        )
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

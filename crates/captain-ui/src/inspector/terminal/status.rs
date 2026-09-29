//! What the Terminal tab shows when there is no live shell.

use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use super::terminal_pane::Phase;
use crate::theme::Palette;

/// The body for a stopped container.
pub fn not_running(palette: &Palette) -> Div {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(8.))
        .p(px(24.))
        .child(
            Icon::new(IconName::SquareTerminal)
                .size(px(28.))
                .text_color(palette.text3),
        )
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .child("Container is not running"),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(palette.text2)
                .text_center()
                .child("Start the container to open a shell in it."),
        )
}

/// A note over the bottom of the grid while the session is connecting or has ended.
pub fn overlay(phase: &Phase, palette: &Palette) -> Option<Div> {
    let (text, color) = match phase {
        Phase::Idle | Phase::Running => return None,
        Phase::Connecting => ("Connecting…".to_string(), palette.text2),
        Phase::Exited(Some(code)) => (
            format!("Process exited with code {code} — Reconnect to start a new shell"),
            palette.text2,
        ),
        Phase::Exited(None) => (
            "Process exited — Reconnect to start a new shell".to_string(),
            palette.text2,
        ),
        Phase::Failed(error) => (format!("Could not start a shell: {error}"), palette.red),
    };
    Some(
        div()
            .absolute()
            .left(px(8.))
            .right(px(8.))
            .bottom(px(8.))
            .px(px(10.))
            .py(px(6.))
            .rounded(px(7.))
            .bg(palette.panel)
            .border_1()
            .border_color(palette.sep)
            .text_size(px(11.))
            .text_color(color)
            .child(text),
    )
}

use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::terminal_pane::{Phase, TerminalPane};
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

/// The bar above the grid: a state dot, the shell and container, the grid size, and
/// Reconnect once the session has ended.
pub fn render(pane: &TerminalPane, palette: &Palette, cx: &mut Context<TerminalPane>) -> Div {
    let name = pane.target.as_ref().map_or("", |t| t.name.as_str());
    let command = pane.command.as_deref().unwrap_or("shell");
    let title = pane
        .emulator
        .title()
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| format!("{command} · {name}"));
    let dot = match pane.phase {
        _ if !pane.running() => palette.gray,
        Phase::Running => palette.green,
        Phase::Connecting | Phase::Idle => palette.orange,
        Phase::Exited(_) | Phase::Failed(_) => palette.red,
    };
    let (cols, rows) = pane.emulator.size();
    let ended = matches!(pane.phase, Phase::Exited(_) | Phase::Failed(_));

    div()
        .h(px(26.))
        .flex()
        .items_center()
        .gap(px(8.))
        .child(div().size(px(7.)).flex_shrink_0().rounded_full().bg(dot))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .font_family(palette.mono())
                .text_size(px(11.))
                .text_color(palette.text2)
                .child(title),
        )
        .when_some(pane.running().then_some(()), |bar, ()| {
            bar.child(
                div()
                    .flex_shrink_0()
                    .font_family(palette.mono())
                    .text_size(px(11.))
                    .text_color(palette.text3)
                    .child(format!("{cols} × {rows}")),
            )
        })
        .when(ended && pane.running(), |bar| {
            bar.child(text_button(
                "terminal-reconnect",
                "Reconnect",
                ButtonTone::Accent,
                true,
                palette,
                cx.listener(|this, _, _, cx| this.reconnect(cx)),
            ))
        })
}

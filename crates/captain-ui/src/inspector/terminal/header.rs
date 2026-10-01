use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::terminal_pane::TerminalPane;
use crate::terminal::Phase;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

/// The bar above the grid: a state dot, the shell and container, the grid size, and
/// Reconnect once the session has ended.
pub fn render(pane: &TerminalPane, palette: &Palette, cx: &mut Context<TerminalPane>) -> Div {
    let view = pane.view.read(cx);
    let name = pane.target.as_ref().map_or("", |t| t.name.as_str());
    let command = view.command().unwrap_or("shell");
    let title = view
        .title()
        .unwrap_or_else(|| format!("{command} · {name}"));
    let dot = match view.phase() {
        _ if !pane.running() => palette.gray,
        Phase::Running => palette.green,
        Phase::Connecting | Phase::Idle => palette.orange,
        Phase::Exited(_) | Phase::Failed(_) => palette.red,
    };
    let (cols, rows) = view.size();
    let ended = matches!(view.phase(), Phase::Exited(_) | Phase::Failed(_));
    let target = pane.view.clone();

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
        .when(pane.running(), |bar| {
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
                move |_, _, cx| target.update(cx, |view, cx| view.restart(cx)),
            ))
        })
}

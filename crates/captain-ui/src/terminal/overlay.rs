//! A bar over the bottom of the grid while the session connects or after it ended.

use gpui_kit::*;

use super::terminal_view::{CloseRequested, EndedBar, Phase, TerminalView};
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

pub fn render(
    view: &TerminalView,
    palette: &Palette,
    cx: &mut Context<TerminalView>,
) -> Option<Div> {
    let buttons = view.ended_bar == EndedBar::Buttons;
    let (text, color) = match &view.phase {
        Phase::Idle | Phase::Running => return None,
        Phase::Connecting => ("Connecting…".to_string(), palette.text2),
        Phase::Exited(code) if buttons => (exited(*code), palette.text2),
        Phase::Exited(code) => (
            format!("{} — Reconnect to start a new shell", exited(*code)),
            palette.text2,
        ),
        Phase::Failed(error) => (format!("Could not start a shell: {error}"), palette.red),
    };
    let ended = matches!(view.phase, Phase::Exited(_) | Phase::Failed(_));
    let bar = div()
        .absolute()
        .left(px(8.))
        .right(px(8.))
        .bottom(px(8.))
        .flex()
        .items_center()
        .gap(px(8.))
        .px(px(10.))
        .py(px(6.))
        .rounded(px(7.))
        .bg(palette.panel)
        .border_1()
        .border_color(palette.sep)
        .text_size(px(11.))
        .text_color(color)
        .child(div().flex_1().min_w_0().child(text));
    if !(buttons && ended) {
        return Some(bar);
    }
    let restart = text_button(
        "terminal-restart",
        "Restart",
        ButtonTone::Accent,
        true,
        palette,
        cx.listener(|this, _, _, cx| {
            this.restart(cx);
            this.focus_next_render(cx);
        }),
    )
    .help("Start a new shell in this tab, in the same folder.");
    let close = text_button(
        "terminal-close-ended",
        "Close tab",
        ButtonTone::Accent,
        true,
        palette,
        cx.listener(|_, _, _, cx| cx.emit(CloseRequested)),
    )
    .help("Close this terminal tab.");
    Some(bar.child(restart).child(close))
}

fn exited(code: Option<i64>) -> String {
    match code {
        Some(code) => format!("Process exited with code {code}"),
        None => "Process exited".to_string(),
    }
}

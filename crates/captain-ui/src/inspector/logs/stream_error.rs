use gpui_kit::*;

use super::LogsPane;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

/// A row with why the log stream stopped, and Reconnect. `None` while it runs.
pub fn render(pane: &LogsPane, palette: &Palette, cx: &mut Context<LogsPane>) -> Option<Div> {
    let error = pane.stream_error()?.to_string();
    Some(
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(px(12.))
                    .text_color(palette.red)
                    .child(error),
            )
            .child(text_button(
                "logs-reconnect",
                "Reconnect",
                ButtonTone::Accent,
                true,
                palette,
                cx.listener(|this, _, _, cx| this.reconnect(cx)),
            )),
    )
}

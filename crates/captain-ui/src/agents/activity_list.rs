//! The Agent activity list: the newest calls agents made to Captain, from the
//! activity log that `captain mcp` writes.

use captain_core::agent_clients::client_label;
use captain_core::agent_tools::Activity;
use chrono::{Local, TimeZone};
use gpui_kit::*;

use crate::help::HelpExt;
use crate::theme::Palette;

/// How many calls the list shows.
const SHOWN: usize = 20;

pub fn render(entries: &[Activity], palette: &Palette) -> Div {
    let rows = entries.iter().take(SHOWN).enumerate().map(|(ix, entry)| {
        let dot = match (entry.ok, entry.is_action()) {
            (false, _) => palette.red,
            (true, true) => palette.orange,
            (true, false) => palette.gray,
        };
        let client = client_label(&entry.client);
        let help = if entry.result.is_empty() {
            format!("{client} called {}.", entry.summary())
        } else {
            format!("{client} called {}: {}", entry.summary(), entry.result)
        };
        div()
            .id(("agents-activity", ix))
            .flex()
            .items_center()
            .gap(px(10.))
            .text_size(px(12.))
            .child(
                div()
                    .w(px(62.))
                    .flex_shrink_0()
                    .font_family(palette.mono())
                    .text_color(palette.text3)
                    .child(time(entry.at)),
            )
            .child(div().size(px(7.)).flex_shrink_0().rounded_full().bg(dot))
            .child(div().w(px(110.)).flex_shrink_0().truncate().child(client))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_color(palette.text2)
                    .child(entry.summary()),
            )
            .help(help)
    });
    let empty = entries.is_empty().then(|| {
        div()
            .text_size(px(12.))
            .text_color(palette.text3)
            .child("No agent has called Captain yet.")
    });
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .px(px(16.))
        .py(px(14.))
        .rounded(px(12.))
        .border_1()
        .border_color(palette.sep)
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .child("Agent activity"),
        )
        .children(empty)
        .children(rows)
}

/// `12:07:11` in local time.
pub fn time(at: i64) -> String {
    Local
        .timestamp_opt(at, 0)
        .single()
        .map(|time| time.format("%H:%M:%S").to_string())
        .unwrap_or_default()
}

//! A row per agent Captain finds, with Connect or Remove, and the step that waits
//! for the user to confirm it: the command, the link, or the changed lines.

use captain_core::agent_clients::{ClientState, ClientStep};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::agents_sheet::AgentsSheet;
use super::clients_state::Pending;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

pub fn render(sheet: &AgentsSheet, palette: &Palette, cx: &mut Context<AgentsSheet>) -> Div {
    let card = &sheet.clients;
    let title = div()
        .font_weight(FontWeight::SEMIBOLD)
        .child("Agents on this computer");
    let body: Vec<AnyElement> = match &card.states {
        None => vec![note("Looking for agents\u{2026}", palette).into_any_element()],
        Some(states) if states.is_empty() => vec![
            note(
                "Captain found no agents. For another agent, use Copy config below.",
                palette,
            )
            .into_any_element(),
        ],
        Some(states) => states
            .iter()
            .map(|state| {
                let pending = card
                    .pending
                    .as_ref()
                    .filter(|pending| pending.client == state.client);
                let preview = pending
                    .zip(card.paths.as_ref())
                    .map(|(pending, paths)| (pending, pending.step.preview(paths)));
                row(state, preview, card.busy, palette, cx).into_any_element()
            })
            .collect(),
    };
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .px(px(16.))
        .py(px(14.))
        .rounded(px(12.))
        .border_1()
        .border_color(palette.sep)
        .child(title)
        .children(body)
        .children(card.error.clone().map(|error| {
            div()
                .text_size(px(12.))
                .text_color(palette.red)
                .child(error)
        }))
        .children(card.captain.is_none().then(|| {
            note(
                "Captain cannot find its captain command, so it cannot connect agents. Install \
                 the command-line tools in Settings > Terminal.",
                palette,
            )
        }))
}

fn row(
    state: &ClientState,
    pending: Option<(&Pending, String)>,
    busy: bool,
    palette: &Palette,
    cx: &mut Context<AgentsSheet>,
) -> Div {
    let name = state.client.name();
    let (status, color, connected) = match &state.connected {
        Ok(true) => ("Connected".to_string(), palette.green, true),
        Ok(false) => ("Not connected".to_string(), palette.text3, false),
        Err(why) => (format!("Captain {why}"), palette.red, false),
    };
    let label = if connected { "Remove" } else { "Connect" };
    let tone = if connected {
        ButtonTone::Danger
    } else {
        ButtonTone::Accent
    };
    let this = cx.entity().downgrade();
    let target = state.clone();
    let button = text_button(
        SharedString::from(format!(
            "agents-{}-{}",
            state.client.id(),
            label.to_lowercase()
        )),
        label,
        tone,
        !busy && pending.is_none(),
        palette,
        move |_, _, cx| {
            this.update(cx, |sheet, cx| sheet.prepare(&target, !connected, cx))
                .ok();
        },
    )
    .help(if connected {
        format!("Remove Captain's server from {name}. Captain shows the step first.")
    } else {
        format!("Add Captain's server to {name}. Captain shows the step first.")
    });
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .id(SharedString::from(format!(
                    "agents-row-{}",
                    state.client.id()
                )))
                .flex()
                .items_center()
                .gap(px(10.))
                .child(div().w(px(140.)).child(name))
                .child(div().size(px(7.)).rounded_full().bg(color))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(12.))
                        .text_color(palette.text2)
                        .child(status),
                )
                .child(button)
                .help(format!("Whether {name} lists Captain's MCP server.")),
        )
        .children(pending.map(|(pending, preview)| confirm(pending, preview, busy, palette, cx)))
}

/// The step, and the buttons that run it or drop it.
fn confirm(
    pending: &Pending,
    preview: String,
    busy: bool,
    palette: &Palette,
    cx: &mut Context<AgentsSheet>,
) -> Div {
    let (lead, run) = match &pending.step {
        ClientStep::Run(_) => ("Captain will run this in your login shell:", "Run"),
        ClientStep::Open(_) => (
            "Captain will open this link, and the agent asks you to confirm:",
            "Open link",
        ),
        ClientStep::Edit { .. } => (
            "Captain will change these lines, and keep a copy of the file first:",
            "Save",
        ),
    };
    let id = pending.client.id();
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .p(px(12.))
        .rounded(px(9.))
        .bg(palette.field)
        .child(
            div()
                .text_size(px(12.))
                .text_color(palette.text2)
                .child(lead),
        )
        .child(
            div()
                .id(SharedString::from(format!("agents-{id}-step")))
                .max_h(px(180.))
                .overflow_y_scroll()
                .px(px(10.))
                .py(px(8.))
                .rounded(px(7.))
                .bg(palette.terminal)
                .font_family(palette.mono())
                .text_size(px(11.5))
                .whitespace_normal()
                .child(preview)
                .help("The exact step Captain takes. Nothing changes until you click the button."),
        )
        .child(
            div()
                .flex()
                .gap(px(8.))
                .justify_end()
                .child(
                    text_button(
                        SharedString::from(format!("agents-{id}-cancel")),
                        "Cancel",
                        ButtonTone::Danger,
                        !busy,
                        palette,
                        cx.listener(|sheet, _, _, cx| {
                            sheet.clients.pending = None;
                            cx.notify();
                        }),
                    )
                    .help("Close this step without changing anything."),
                )
                .child(
                    text_button(
                        SharedString::from(format!("agents-{id}-run")),
                        run,
                        ButtonTone::Accent,
                        !busy,
                        palette,
                        cx.listener(|sheet, _, window, cx| sheet.run_pending(window, cx)),
                    )
                    .help("Take the step shown above."),
                ),
        )
        .when(busy, |confirm| confirm.opacity(0.6))
}

fn note(text: &'static str, palette: &Palette) -> Div {
    div()
        .text_size(px(12.))
        .text_color(palette.text3)
        .child(text)
}

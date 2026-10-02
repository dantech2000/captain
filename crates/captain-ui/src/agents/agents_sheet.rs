//! The AI agents sheet, from Settings: the switch that lets agents use Captain,
//! the actions they may run, a row per agent Captain finds with Connect or Remove,
//! Copy config, and the Agent activity list. See docs/features/0038-agent-tools.md.

use captain_core::agent_clients::copy_config;
use captain_core::agent_tools::{AgentAction, AgentToolsSettings};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable, WindowExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::activity_watch::agent_activity;
use super::clients_state::ClientsCard;
use super::{activity_list, client_rows};
use crate::help::HelpExt;
use crate::settings::{current, update};
use crate::theme::Palette;
use crate::widgets::primary_button;

pub struct AgentsSheet {
    pub(super) clients: ClientsCard,
    _subscriptions: Vec<Subscription>,
}

/// Opens the sheet and looks for agents.
pub fn open(window: &mut Window, cx: &mut App) {
    let sheet = cx.new(|cx: &mut Context<AgentsSheet>| {
        let mut subscriptions =
            vec![cx.observe_global::<crate::settings::SettingsStore>(|_, cx| cx.notify())];
        subscriptions
            .extend(agent_activity(cx).map(|watch| cx.observe(&watch, |_, _, cx| cx.notify())));
        let mut sheet = AgentsSheet {
            clients: ClientsCard::new(),
            _subscriptions: subscriptions,
        };
        sheet.find_clients(cx);
        sheet
    });
    window.open_dialog(cx, move |dialog, _, _| {
        dialog.w(px(640.)).margin_top(px(90.)).child(sheet.clone())
    });
}

impl Render for AgentsSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let entries = agent_activity(cx)
            .map(|watch| watch.read(cx).entries().to_vec())
            .unwrap_or_default();
        div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .child(header(&palette))
            .child(
                div()
                    .id("agents-sheet-body")
                    .max_h(px(520.))
                    .overflow_y_scrollbar()
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .child(access(current(cx).agent_tools, &palette))
                    .child(client_rows::render(self, &palette, cx))
                    .child(activity_list::render(&entries, &palette)),
            )
            .child(self.footer(&palette, cx))
    }
}

fn header(palette: &Palette) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(4.))
        // Leave room for the dialog's close button in the corner.
        .pr(px(32.))
        .child(
            div()
                .text_size(px(18.))
                .font_weight(FontWeight::EXTRA_BOLD)
                .child("Connect AI agents"),
        )
        .child(div().text_color(palette.text2).child(
            "Agents such as Claude Code read Captain's projects, containers, and logs \
             through captain mcp.",
        ))
}

/// The switch and the actions.
fn access(settings: AgentToolsSettings, palette: &Palette) -> Div {
    let enabled = settings.enabled;
    let actions = AgentAction::ALL.into_iter().map(move |action| {
        let checked = settings.actions.contains(&action);
        div()
            .id(SharedString::from(format!(
                "agents-action-{}",
                action.name()
            )))
            .child(
                Checkbox::new(SharedString::from(format!(
                    "agents-check-{}",
                    action.name()
                )))
                .label(action.label())
                .checked(checked)
                .disabled(!enabled)
                .on_click(move |checked, _, cx| {
                    let checked = *checked;
                    update(cx, |settings| {
                        settings.agent_tools.set_allowed(action, checked);
                    });
                }),
            )
            .help(action_help(action))
    });
    div()
        .flex()
        .flex_col()
        .gap(px(10.))
        .px(px(16.))
        .py(px(14.))
        .rounded(px(12.))
        .border_1()
        .border_color(palette.sep)
        .child(
            div()
                .id("agents-enabled-row")
                .flex()
                .items_center()
                .gap(px(12.))
                .child(
                    div()
                        .flex_1()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Let agents use Captain"),
                )
                .child(
                    Switch::new("agents-enabled")
                        .checked(enabled)
                        .on_click(|checked, _, cx| {
                            let checked = *checked;
                            update(cx, |settings| settings.agent_tools.enabled = checked);
                        }),
                )
                .help("Let connected agents read the engine, projects, containers, problems, logs, and disk use."),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(palette.text3)
                .child(if enabled {
                    "Agents can read, and run only the actions checked below. They never remove \
                     anything, run other commands, or change settings."
                } else {
                    "Off: agents see only the help tool, which says how to turn this on."
                }),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(14.))
                .when(!enabled, |row| row.opacity(0.5))
                .children(actions),
        )
}

fn action_help(action: AgentAction) -> &'static str {
    match action {
        AgentAction::Start => "Let agents start a container or a Compose project.",
        AgentAction::Stop => {
            "Let agents stop a container or a Compose project. Nothing is removed."
        }
        AgentAction::Restart => "Let agents restart a container or a Compose project.",
        AgentAction::RunTask => {
            "Let agents run the tasks your Compose files declare in x-captain.tasks, and only those."
        }
        AgentAction::RaiseMemory => {
            "Let agents raise a container's memory limit: twice the old one, at least 512 MB."
        }
    }
}

impl AgentsSheet {
    fn footer(&self, palette: &Palette, cx: &mut Context<Self>) -> Div {
        let captain = self.clients.captain.clone();
        div()
            .flex()
            .items_center()
            .gap(px(10.))
            .child(
                div()
                    .id("agents-copy-config")
                    .text_size(px(12.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.link)
                    .cursor_pointer()
                    .on_click(move |_, window, cx| {
                        let Some(captain) = &captain else {
                            return;
                        };
                        cx.write_to_clipboard(ClipboardItem::new_string(copy_config(captain)));
                        window.push_notification(Notification::success("Copied the MCP config."), cx);
                    })
                    .child("Copy config")
                    .help("Copy the mcpServers JSON that runs captain mcp, for an agent that is not listed."),
            )
            .child(div().flex_1())
            .child(
                div()
                    .id("agents-check-again")
                    .h(px(32.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .rounded(px(8.))
                    .border_1()
                    .border_color(palette.border_strong)
                    .text_size(px(12.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .cursor_pointer()
                    .hover(|style| style.opacity(0.85))
                    .on_click(cx.listener(|sheet, _, _, cx| sheet.find_clients(cx)))
                    .child("Check again")
                    .help("Look again for agents and whether each one lists Captain."),
            )
            .child(primary_button(
                "agents-sheet-done",
                "Done",
                "Close the AI agents sheet.",
                true,
                palette,
                |_, window, cx| window.close_dialog(cx),
            ))
    }
}

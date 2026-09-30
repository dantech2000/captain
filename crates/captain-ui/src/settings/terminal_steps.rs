//! The steps of the terminal setup sheet: link the tools, and make docker use
//! Captain Engine. PATH is in `path_step.rs`, the socket link in `admin_access.rs`.

use captain_core::cli_tools::{self, LinkState};
use captain_core::settings::Settings;
use gpui_kit::assets::IconName;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Icon, WindowExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::cli_tools_state::TerminalFacts;
use super::{SettingsView, context_actions, store};
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

/// Where a step stands: done, the next one to do, or later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Done,
    Next,
    Later,
}

/// One step: its number or a check, its title, what it needs, and its button.
pub fn step(
    number: usize,
    state: StepState,
    title: impl Into<SharedString>,
    body: impl IntoElement,
    action: Option<AnyElement>,
    palette: &Palette,
) -> Div {
    let title: SharedString = title.into();
    let badge = div()
        .size(px(22.))
        .flex_shrink_0()
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(11.))
        .font_weight(FontWeight::EXTRA_BOLD);
    let badge = match state {
        StepState::Done => badge.bg(palette.green).child(
            Icon::new(IconName::Check)
                .size(px(12.))
                .text_color(palette.bg),
        ),
        StepState::Next => badge
            .border_2()
            .border_color(palette.accent)
            .text_color(palette.link)
            .child(number.to_string()),
        StepState::Later => badge
            .border_2()
            .border_color(palette.border_strong)
            .text_color(palette.text3)
            .child(number.to_string()),
    };
    div()
        .flex()
        .gap(px(12.))
        .px(px(16.))
        .py(px(14.))
        .when(state == StepState::Next, |row| {
            row.bg(palette.accent.alpha(0.06))
        })
        .child(badge)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
                .child(body),
        )
        .children(action.map(|action| div().flex_shrink_0().child(action)))
}

/// A quiet sentence under a step's title.
pub fn step_note(text: impl Into<SharedString>, palette: &Palette) -> Div {
    let text: SharedString = text.into();
    div()
        .text_size(px(12.))
        .text_color(palette.text3)
        .child(text)
}

/// A line of code with a Copy button.
pub fn code_box(id: &'static str, code: &'static str, palette: &Palette) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .px(px(12.))
        .py(px(10.))
        .rounded(px(9.))
        .bg(palette.terminal)
        .border_1()
        .border_color(palette.sep)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .font_family(palette.mono())
                .text_size(px(12.))
                .child(code),
        )
        .child(
            text_button(
                id,
                "Copy",
                ButtonTone::Accent,
                true,
                palette,
                move |_, window, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(code.to_string()));
                    window.push_notification(Notification::success("Copied."), cx);
                },
            )
            .help(format!("Copy {code}")),
        )
}

pub fn state(done: bool, next: bool) -> StepState {
    match (done, next) {
        (true, _) => StepState::Done,
        (false, true) => StepState::Next,
        (false, false) => StepState::Later,
    }
}

/// Step 1: the links in `~/.captain/bin` and the plugin folder, with Relink.
pub fn links(
    view: &SettingsView,
    this: &WeakEntity<SettingsView>,
    settings: &Settings,
    facts: &TerminalFacts,
    palette: &Palette,
) -> Div {
    let card = &view.cli_tools;
    let links = card.status.as_ref().map(|status| &status.links);
    let note = match links {
        Some(links) if links.is_empty() => {
            "This Captain does not run from Captain.app, so there are no tools to link.".into()
        }
        Some(links) if settings.command_line_tools.enabled => {
            let wrong: Vec<String> = links
                .iter()
                .filter(|report| !matches!(report.state, LinkState::Linked | LinkState::NoTarget))
                .filter_map(|report| Some(report.link.path.file_name()?.to_string_lossy().into()))
                .collect();
            match wrong.is_empty() {
                true => "In ~/.captain/bin, and the plugins in ~/.docker/config.json.".to_string(),
                false => format!("Not linked: {}.", wrong.join(", ")),
            }
        }
        _ => "Captain links its tools into ~/.captain/bin, and adds its plugin folder to ~/.docker/config.json.".to_string(),
    };
    let enabled = settings.command_line_tools.enabled;
    let can_link = !card.busy && links.is_some_and(|links| !links.is_empty());
    let tools = settings.command_line_tools.clone();
    let this = this.clone();
    let button = text_button(
        "settings-tools-relink",
        match (card.busy, enabled) {
            (true, _) => "Working\u{2026}",
            (false, true) => "Relink",
            (false, false) => "Link",
        },
        ButtonTone::Accent,
        can_link,
        palette,
        move |_, _, cx| {
            let tools = cli_tools::CliToolsSettings {
                enabled: true,
                ..tools.clone()
            };
            store::update(cx, |settings| settings.command_line_tools = tools.clone());
            this.update(cx, |view, cx| view.install_tools(tools, cx)).ok();
        },
    )
    .help(if enabled {
        "Link the tools again, for example after you moved Captain.app."
    } else {
        "Link docker, Compose, the keychain helper, kubectl, and helm into ~/.captain/bin, and add Captain's plugin folder to ~/.docker/config.json."
    });
    step(
        1,
        state(facts.steps.links, true),
        "Link the tools",
        step_note(note, palette),
        Some(button.into_any_element()),
        palette,
    )
}

/// Step 3: make `captain-engine` the docker CLI's default context.
pub fn context(
    view: &SettingsView,
    this: &WeakEntity<SettingsView>,
    facts: &TerminalFacts,
    palette: &Palette,
) -> Div {
    let done = facts.steps.context;
    let note = match (&facts.socket, done) {
        (None, _) => "Captain Engine is not set up, so there is no captain-engine context to use."
            .to_string(),
        (Some(_), true) => "New docker commands use the captain-engine context.".into(),
        (Some(_), false) => format!(
            "Now: {}. Captain makes the captain-engine context the default.",
            facts.current
        ),
    };
    let (socket, points_here, this) = (facts.socket.clone(), facts.points_here, this.clone());
    let button = (!done).then(|| {
        text_button(
            "settings-tools-use-captain",
            "Use Captain Engine",
            ButtonTone::Accent,
            socket.is_some() && !view.context_change.busy,
            palette,
            move |_, window, cx| {
                if let Some(socket) = socket.clone() {
                    context_actions::use_captain(this.clone(), socket, !points_here, window, cx);
                }
            },
        )
        .help("Make the captain-engine context the docker CLI's default, and create it first if needed.")
        .into_any_element()
    });
    let next = facts.steps.links && facts.steps.path;
    step(
        3,
        state(done, next),
        "Make docker use Captain Engine",
        step_note(note, palette),
        button,
        palette,
    )
}

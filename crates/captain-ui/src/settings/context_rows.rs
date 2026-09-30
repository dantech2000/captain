//! The Docker CLI contexts in the engines sheet: one row per context, and a row
//! that creates the `captain` context for Captain Engine. See
//! docs/features/0026-contexts-and-remote-hosts.md.

use captain_core::docker_context::{CAPTAIN_CONTEXT, DockerContext};
use gpui_kit::*;

use super::listen::listen;
use super::{SettingsView, context_actions};
use crate::engine_host::captain_socket;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, settings_row, text_button};

/// The context rows. `in_use` is the endpoint Captain is connected to.
pub fn rows(
    view: &SettingsView,
    this: &WeakEntity<SettingsView>,
    in_use: Option<&str>,
    palette: &Palette,
    cx: &App,
) -> Vec<AnyElement> {
    let mut rows: Vec<AnyElement> = view
        .contexts
        .contexts
        .iter()
        .enumerate()
        .map(|(ix, context)| {
            let default = view.contexts.is_current(&context.name);
            context_row(ix, context, default, in_use, view, this, palette)
        })
        .collect();
    rows.extend(captain_row(view, this, palette, cx));
    if let Some(error) = view.context_change.error.clone() {
        rows.push(
            div()
                .px(px(14.))
                .py(px(8.))
                .text_size(px(11.))
                .text_color(palette.red)
                .child(error)
                .into_any_element(),
        );
    }
    rows
}

fn context_row(
    ix: usize,
    context: &DockerContext,
    default: bool,
    in_use: Option<&str>,
    view: &SettingsView,
    this: &WeakEntity<SettingsView>,
    palette: &Palette,
) -> AnyElement {
    let host = context.host.clone();
    let active = host.is_some() && host.as_deref() == in_use;
    let mut note: Vec<&str> = vec![context.host.as_deref().unwrap_or("No Docker endpoint")];
    if !context.description.is_empty() {
        note.push(&context.description);
    }
    if default {
        note.push("Default context");
    }
    let use_button = text_button(
        ("context-use", ix),
        if active { "In use" } else { "Use" },
        ButtonTone::Accent,
        !active && host.is_some(),
        palette,
        listen(this, move |view, _, cx| view.use_engine(host.clone(), cx)),
    );
    let name = context.name.clone();
    let this = this.clone();
    let default_button = (!default).then(|| {
        text_button(
            ("context-default", ix),
            "Make default\u{2026}",
            ButtonTone::Accent,
            !view.context_change.busy,
            palette,
            move |_, window, cx| {
                context_actions::make_default(this.clone(), name.clone(), window, cx)
            },
        )
    });
    settings_row(
        div()
            .font_family(palette.mono())
            .text_size(px(12.))
            .child(context.name.clone()),
        Some(note.join(" \u{00b7} ").into()),
        div()
            .flex()
            .gap(px(8.))
            .children(default_button)
            .child(use_button),
        palette,
    )
    .id(("settings-context", ix))
    .help(format!(
        "Use the {} context, or make it the docker CLI's default.",
        context.name
    ))
    .into_any_element()
}

/// Create context, or Update context when `captain` points elsewhere. `None` when
/// there is no Captain Engine, or the context already points at it.
fn captain_row(
    view: &SettingsView,
    this: &WeakEntity<SettingsView>,
    palette: &Palette,
    cx: &App,
) -> Option<AnyElement> {
    let socket = captain_socket(cx)?;
    let existing = view.contexts.get(CAPTAIN_CONTEXT);
    if existing.is_some_and(|context| context.host.as_deref() == Some(socket.as_str())) {
        return None;
    }
    let update = existing.is_some();
    let this = this.clone();
    let button = text_button(
        "context-create-captain",
        if update {
            "Update context\u{2026}"
        } else {
            "Create context\u{2026}"
        },
        ButtonTone::Accent,
        !view.context_change.busy,
        palette,
        move |_, window, cx| {
            context_actions::create_captain(this.clone(), socket.clone(), update, window, cx)
        },
    );
    let note = format!("A \"{CAPTAIN_CONTEXT}\" context lets the docker CLI use Captain Engine.");
    Some(
        settings_row("Captain Engine context", Some(note.into()), button, palette)
            .id("settings-captain-context").help(format!("Create or update the \"{CAPTAIN_CONTEXT}\" context, so the docker CLI uses Captain Engine.")).into_any_element(),
    )
}

//! Create context and Make default: they change the user's Docker CLI state, so each
//! asks first. See docs/features/0026-contexts-and-remote-hosts.md.

use captain_core::docker_context::CAPTAIN_CONTEXT;
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::SettingsView;
use super::engine_source::{ContextJob, engine_source};

/// A context change in progress, or the error of the last one.
#[derive(Default)]
pub struct ContextChange {
    pub busy: bool,
    pub error: Option<SharedString>,
}

/// Asks, then creates or updates the `captain` context to point at `host`.
pub fn create_captain(
    view: WeakEntity<SettingsView>,
    host: String,
    update: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let (title, ok) = match update {
        true => ("Update the captain context?", "Update"),
        false => ("Create a captain context?", "Create"),
    };
    let description = format!(
        "Captain runs \"docker context {} {CAPTAIN_CONTEXT}\" with the host {host}. \
         Other contexts do not change, and the default context stays the same.",
        if update { "update" } else { "create" },
    );
    window.open_alert_dialog(cx, move |alert, _, _| {
        let (view, host) = (view.clone(), host.clone());
        alert
            .title(title)
            .description(description.clone())
            .show_cancel(true)
            .ok_text(ok)
            .on_ok(move |_, _, cx| {
                let job = engine_source(cx)
                    .map(|source| source.save_context(CAPTAIN_CONTEXT, "Captain Engine", &host));
                run(view.clone(), job, cx);
                true
            })
    });
}

/// Asks, then makes `name` the Docker CLI's default context.
pub fn make_default(
    view: WeakEntity<SettingsView>,
    name: String,
    window: &mut Window,
    cx: &mut App,
) {
    let description = format!(
        "Captain runs \"docker context use {name}\". The docker CLI and other tools that \
         read your Docker config then use this context."
    );
    window.open_alert_dialog(cx, move |alert, _, _| {
        let (view, name) = (view.clone(), name.clone());
        alert
            .title(format!("Make {name} the default context?"))
            .description(description.clone())
            .show_cancel(true)
            .ok_text("Make default")
            .on_ok(move |_, _, cx| {
                let job = engine_source(cx).map(|source| source.use_context(&name));
                run(view.clone(), job, cx);
                true
            })
    });
}

/// Runs `job` in the background, then reads the contexts again.
fn run(view: WeakEntity<SettingsView>, job: Option<ContextJob>, cx: &mut App) {
    let Some(job) = job else {
        return;
    };
    let started = view.update(cx, |view, cx| {
        view.context_change = ContextChange {
            busy: true,
            error: None,
        };
        cx.notify();
    });
    if started.is_err() {
        return;
    }
    cx.spawn(async move |cx| {
        let result = cx.background_executor().spawn(async move { job() }).await;
        if let Err(error) = &result {
            tracing::warn!(%error, "cannot change the Docker contexts");
        }
        view.update(cx, |view, cx| {
            view.context_change = ContextChange {
                busy: false,
                error: result.err().map(Into::into),
            };
            view.rescan(cx);
        })
        .ok();
    })
    .detach();
}

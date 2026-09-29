//! The Administrative access card: links `/var/run/docker.sock` to Captain Engine's
//! socket, so tools that use the default socket reach Captain Engine. See feature 0015.

use std::path::{Path, PathBuf};

use captain_core::behavior::docker_socket::{DEFAULT_SOCKET, LinkAction, SocketLink, socket_path};
use gpui_kit::component::WindowExt;
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::*;

use super::{SettingsView, system};
use crate::engine_host::captain_endpoint;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, settings_card, settings_row, text_button};

/// A link or unlink in progress, or the error of the last one.
#[derive(Default)]
pub struct AdminAccess {
    busy: bool,
    error: Option<SharedString>,
}

/// The card, or `None` when Captain Engine is not in use, has no Unix socket, or
/// already listens on the default socket.
pub fn render(
    view: &SettingsView,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> Option<Div> {
    let system = system::system(cx)?;
    let captain = captain_endpoint(cx).and_then(|endpoint| socket_path(&endpoint))?;
    if captain == Path::new(DEFAULT_SOCKET) {
        return None;
    }
    let link = SocketLink::classify(&system.docker_socket()?, &captain);
    let admin = &view.admin_access;
    let note = admin.error.clone().unwrap_or_else(|| match link.action() {
        Some(_) => format!("{} Changing it asks for your password.", link.describe()).into(),
        None => link.describe().into(),
    });
    let label = match (admin.busy, link.action()) {
        (true, _) => "Waiting for approval\u{2026}",
        (false, Some(LinkAction::Unlink)) => "Remove link\u{2026}",
        (false, _) => "Link to Captain Engine\u{2026}",
    };
    let action = link.action().filter(|_| !admin.busy);
    let this = cx.weak_entity();
    let button = text_button(
        "admin-access-link",
        label,
        ButtonTone::Accent,
        action.is_some(),
        palette,
        move |_, window, cx| {
            let Some(action) = action else {
                return;
            };
            if link.needs_confirmation() {
                confirm(this.clone(), &link, captain.clone(), window, cx);
            } else {
                run(this.clone(), action, link.clone(), captain.clone(), cx);
            }
        },
    );
    Some(settings_card(
        "Administrative access",
        [settings_row("Default Docker socket", Some(note), button, palette).into_any_element()],
        palette,
    ))
}

/// Asks before Captain replaces another engine's socket.
fn confirm(
    view: WeakEntity<SettingsView>,
    link: &SocketLink,
    captain: PathBuf,
    window: &mut Window,
    cx: &mut App,
) {
    let description = format!(
        "{} Tools that use {DEFAULT_SOCKET} will reach Captain Engine instead. \
         The other engine keeps running.",
        link.describe()
    );
    let link = link.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        let view = view.clone();
        let link = link.clone();
        let captain = captain.clone();
        alert
            .title("Replace the Docker socket?")
            .description(description.clone())
            .show_cancel(true)
            .ok_text("Replace")
            .ok_variant(ButtonVariant::Danger)
            .on_ok(move |_, _, cx| {
                run(
                    view.clone(),
                    LinkAction::Link,
                    link.clone(),
                    captain.clone(),
                    cx,
                );
                true
            })
    });
}

/// Runs the privileged command in the background; it waits for the password prompt.
/// The command acts only if the socket is still `seen`, the state the user acted on.
fn run(
    view: WeakEntity<SettingsView>,
    action: LinkAction,
    seen: SocketLink,
    captain: PathBuf,
    cx: &mut App,
) {
    let Some(system) = system::system(cx) else {
        return;
    };
    let started = view.update(cx, |view, cx| {
        view.admin_access = AdminAccess {
            busy: true,
            error: None,
        };
        cx.notify();
    });
    if started.is_err() {
        return;
    }
    cx.spawn(async move |cx| {
        let result = cx
            .background_executor()
            .spawn(async move {
                match action {
                    LinkAction::Link => system.link_docker_socket(&captain, &seen),
                    LinkAction::Unlink => system.unlink_docker_socket(&captain),
                }
            })
            .await;
        if let Err(error) = &result {
            tracing::warn!(%error, "cannot change {DEFAULT_SOCKET}");
        }
        view.update(cx, |view, cx| {
            view.admin_access = AdminAccess {
                busy: false,
                error: result.err().map(Into::into),
            };
            cx.notify();
        })
        .ok();
    })
    .detach();
}

//! Brings menu clicks from `muda` into GPUI, and runs the command behind each one.

use captain_core::kubernetes::{use_context, user_kubeconfig_paths};
use futures::StreamExt;
use futures::channel::mpsc;
use gpui_kit::*;
use muda::{MenuEvent, MenuId};
use tray_icon::TrayIconEvent;

use super::menu_model::TrayCommand;
use super::raise::raise_memory;
use crate::{quit, window};

/// Forwards menu clicks to a GPUI task on the main thread. `command` looks up what
/// a clicked item does in the current menu.
///
/// `muda` calls the handler on the main thread while the platform run loop handles
/// the click; the channel wakes the GPUI task, so nothing polls. See ADR 0006.
pub fn listen(command: fn(&MenuId, &App) -> Option<TrayCommand>, cx: &mut App) -> Task<()> {
    let (sender, mut receiver) = mpsc::unbounded();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        sender.unbounded_send(event.id).ok();
    }));
    // The menu opens by itself on a click. Drop the icon's clicks, hovers, and
    // moves, so they do not pile up in tray-icon's channel.
    TrayIconEvent::set_event_handler(Some(|_| {}));

    cx.spawn(async move |cx| {
        while let Some(id) = receiver.next().await {
            cx.update(|cx| {
                if let Some(command) = command(&id, cx) {
                    run(command, cx);
                }
            });
        }
    })
}

fn run(command: TrayCommand, cx: &mut App) {
    match command {
        TrayCommand::OpenCaptain => window::show(cx),
        TrayCommand::Settings => window::show_settings(cx),
        TrayCommand::Quit => quit::quit(cx),
        TrayCommand::StartEngine => {
            if let Some(host) = captain_ui::host_model(cx) {
                host.update(cx, |host, cx| host.start(cx));
            }
        }
        TrayCommand::StopEngine => {
            if let Some(host) = captain_ui::host_model(cx) {
                host.update(cx, |host, cx| host.stop(cx)).detach();
            }
        }
        TrayCommand::Container { id, action } => {
            window::workspace(cx).update(cx, |workspace, cx| {
                workspace.run_action(id, action, cx);
            });
        }
        TrayCommand::Containers { ids, action } => {
            window::workspace(cx).update(cx, |workspace, cx| {
                workspace.run_actions(ids, action, cx);
            });
        }
        TrayCommand::FloatLog { id, name } => {
            captain_ui::open_float_log(window::workspace(cx), id, name, cx);
        }
        TrayCommand::RaiseMemory { id, name, bytes } => raise_memory(id, name, bytes, cx),
        // A fix can report in a notification, so it runs in the main window.
        TrayCommand::RunFix(fix) => {
            window::show(cx);
            window::update_main(cx, |window, cx| {
                captain_ui::run_suggested_fix(&fix, window, cx)
            });
        }
        TrayCommand::OpenUrl(url) => cx.open_url(&url),
        TrayCommand::CopyAddress(address) => {
            cx.write_to_clipboard(ClipboardItem::new_string(address));
        }
        // As the Settings switch, then Apply, so the cluster starts or stops now.
        TrayCommand::SetKubernetes(on) => {
            if let Some(model) = captain_ui::kubernetes_model(cx) {
                model.update(cx, |model, cx| {
                    model.turn_on(on, cx);
                    model.apply(cx);
                });
            }
        }
        TrayCommand::UseContext(name) => {
            let paths = user_kubeconfig_paths();
            if let Err(error) = use_context(&paths, &name) {
                tracing::warn!(%error, "cannot switch the Kubernetes context");
            }
            super::controller::refresh_contexts(cx);
        }
    }
}

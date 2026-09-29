//! Brings menu clicks from `muda` into GPUI, and runs the command behind each one.

use futures::StreamExt;
use futures::channel::mpsc;
use gpui_kit::*;
use muda::{MenuEvent, MenuId};
use tray_icon::TrayIconEvent;

use super::menu_model::TrayCommand;
use crate::window;

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
    // Captain does not use icon clicks, hovers, or moves. Without a handler they would
    // pile up in tray-icon's channel for as long as the app runs.
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
        TrayCommand::Quit => cx.quit(),
        TrayCommand::Container { id, action } => {
            window::workspace(cx).update(cx, |workspace, cx| {
                workspace.run_action(id, action, cx);
            });
        }
        TrayCommand::Project { ids, action } => {
            window::workspace(cx).update(cx, |workspace, cx| {
                workspace.run_actions(ids, action, cx);
            });
        }
        TrayCommand::OpenPort(port) => cx.open_url(&format!("http://localhost:{port}")),
    }
}

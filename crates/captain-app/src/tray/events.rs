//! Brings menu clicks from `muda` into GPUI, and runs the command behind each one.

use captain_core::kubernetes::{use_context, user_kubeconfig_paths};
use futures::StreamExt;
use futures::channel::mpsc;
use gpui_kit::*;
use muda::{MenuEvent, MenuId};
use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent};

use super::menu_model::TrayCommand;
use super::placement::Rect;
use crate::{quit, window};

/// Forwards menu clicks and left clicks on the icon to GPUI tasks on the main
/// thread. `command` looks up what a clicked item does in the current menu;
/// `clicked` gets the icon's rectangle in physical pixels.
///
/// `muda` and `tray-icon` call the handlers on the main thread while the platform
/// run loop handles the click; the channels wake the GPUI tasks, so nothing polls.
/// See ADR 0006.
pub fn listen(
    command: fn(&MenuId, &App) -> Option<TrayCommand>,
    clicked: fn(Rect, &mut App),
    cx: &mut App,
) -> [Task<()>; 2] {
    let (sender, mut receiver) = mpsc::unbounded();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        sender.unbounded_send(event.id).ok();
    }));
    // Only the release of a left click counts; hovers and moves are dropped here, so
    // they do not pile up in tray-icon's channel.
    let (click_sender, mut clicks) = mpsc::unbounded();
    TrayIconEvent::set_event_handler(Some(move |event| {
        if let TrayIconEvent::Click {
            rect,
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let rect = Rect {
                x: rect.position.x as f32,
                y: rect.position.y as f32,
                width: rect.size.width as f32,
                height: rect.size.height as f32,
            };
            click_sender.unbounded_send(rect).ok();
        }
    }));

    let menu = cx.spawn(async move |cx| {
        while let Some(id) = receiver.next().await {
            cx.update(|cx| {
                if let Some(command) = command(&id, cx) {
                    run(command, cx);
                }
            });
        }
    });
    let icon = cx.spawn(async move |cx| {
        while let Some(rect) = clicks.next().await {
            cx.update(|cx| clicked(rect, cx));
        }
    });
    [menu, icon]
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
        TrayCommand::Project { ids, action } => {
            window::workspace(cx).update(cx, |workspace, cx| {
                workspace.run_actions(ids, action, cx);
            });
        }
        TrayCommand::OpenPort(port) => cx.open_url(&format!("http://localhost:{port}")),
        TrayCommand::UseContext(name) => {
            let paths = user_kubeconfig_paths();
            if let Err(error) = use_context(&paths, &name) {
                tracing::warn!(%error, "cannot switch the Kubernetes context");
            }
            super::controller::refresh_contexts(cx);
        }
    }
}

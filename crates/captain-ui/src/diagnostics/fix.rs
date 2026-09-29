use std::path::PathBuf;

use captain_core::diagnostics::Fix;
use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use crate::engine_host::host_model;

/// Runs a fix. The host actions are the same ones the sidebar and Settings use.
pub fn run(fix: &Fix, engine_dir: Option<&PathBuf>, window: &mut Window, cx: &mut App) {
    match fix {
        Fix::StartEngine => {
            if let Some(host) = host_model(cx) {
                host.update(cx, |host, cx| host.start(cx));
            }
        }
        Fix::RestartEngine => {
            if let Some(host) = host_model(cx) {
                host.update(cx, |host, cx| host.restart(cx));
            }
        }
        Fix::ShowEngineFiles => {
            if let Some(dir) = engine_dir {
                cx.open_with_system(dir);
            }
        }
        Fix::CopyCommand(command) => {
            cx.write_to_clipboard(ClipboardItem::new_string(command.to_string()));
            let message = format!("Copied \"{command}\". Paste it into a terminal.");
            window.push_notification(Notification::success(message), cx);
        }
    }
}

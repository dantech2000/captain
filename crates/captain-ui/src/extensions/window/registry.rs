//! One window per extension: Open brings an open one forward.

use std::collections::HashMap;
use std::sync::Arc;

use captain_core::extension::{ExtensionManager, InstalledExtension};
use gpui_kit::*;

use super::super::MainWindow;
use super::extension_window::ExtensionWindow;

pub const CAN_OPEN: bool = true;

/// The open extension windows by extension ID. A handle goes stale when the user
/// closes the window.
#[derive(Default)]
struct Windows(HashMap<String, AnyWindowHandle>);

impl Global for Windows {}

/// Opens the extension's window, or brings it to the front.
pub fn open_window(
    extension: InstalledExtension,
    manager: Arc<dyn ExtensionManager>,
    main: MainWindow,
    cx: &mut App,
) {
    let handle = cx.default_global::<Windows>().0.get(&extension.id).copied();
    let activated = handle.is_some_and(|handle| {
        handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    });
    if activated {
        return;
    }
    let bounds = Bounds::centered(None, size(px(1100.), px(760.)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some(extension.title().to_string().into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(14.), px(13.))),
        }),
        window_min_size: Some(size(px(480.), px(320.))),
        app_id: Some("dev.captain.Captain".into()),
        ..Default::default()
    };
    let id = extension.id.clone();
    let opened = gpui_kit::open_window(options, cx, |window, cx| {
        cx.new(|cx| ExtensionWindow::new(extension, manager, main, window, cx))
    });
    match opened {
        Ok((handle, _)) => {
            cx.default_global::<Windows>().0.insert(id, handle);
        }
        Err(error) => tracing::error!(%error, "cannot open the extension window"),
    }
}

/// Closes the extension's window, before Captain removes the extension.
pub fn close_window(id: &str, cx: &mut App) {
    if let Some(handle) = cx.default_global::<Windows>().0.remove(id) {
        handle
            .update(cx, |_, window, _| window.remove_window())
            .ok();
    }
}

/// Closes every extension window, when Captain connects to another engine: each
/// window talks to the engine it was opened on.
pub fn close_all_windows(cx: &mut App) {
    let handles: Vec<AnyWindowHandle> = cx
        .default_global::<Windows>()
        .0
        .drain()
        .map(|(_, handle)| handle)
        .collect();
    for handle in handles {
        handle
            .update(cx, |_, window, _| window.remove_window())
            .ok();
    }
}

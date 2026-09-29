//! The main window, and the workspace that outlives it.

use captain_ui::{AppShell, DiagnosticsSetup, Workspace};
use gpui_kit::*;

use crate::connect;
use crate::engine::EngineSetup;

/// The app owns the workspace, not the window. Closing the window keeps the engine
/// connection, so the menu bar icon still works and a new window starts where the
/// old one stopped.
struct MainWindow {
    workspace: Entity<Workspace>,
    /// The open main window. It is stale after the user closes the window.
    handle: Option<AnyWindowHandle>,
}

impl Global for MainWindow {}

/// Sets up Captain Engine and connects the workspace. The caller opens the window
/// with [`show`], unless Captain starts in the background.
/// `endpoint` is the other engine saved in the settings; `None` means discovery.
/// With Captain Engine, the workspace connects once the engine runs.
pub fn init(
    endpoint: Option<String>,
    engine: EngineSetup,
    diagnostics: DiagnosticsSetup,
    cx: &mut App,
) {
    let connect_now = engine.connects_elsewhere();
    let workspace = cx.new(|cx| {
        let mut workspace = Workspace::new();
        if connect_now {
            workspace.connect(connect::docker(endpoint), cx);
        }
        workspace
    });
    engine.install(&workspace, cx);
    captain_ui::diagnostics_init(cx, &workspace, diagnostics);
    cx.set_global(MainWindow {
        workspace,
        handle: None,
    });
}

/// The workspace that the window and the menu bar icon share.
pub fn workspace(cx: &App) -> Entity<Workspace> {
    cx.global::<MainWindow>().workspace.clone()
}

/// Brings the main window to the front, or opens a new one if the user closed it.
pub fn show(cx: &mut App) {
    let handle = cx.global::<MainWindow>().handle;
    let activated = handle.is_some_and(|handle| {
        handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    });
    if !activated {
        open(cx);
    }
    cx.activate(true);
}

/// Shows the main window on the Settings page. Only the menu bar uses it, and Linux has
/// no menu bar icon (ADR 0006).
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn show_settings(cx: &mut App) {
    show(cx);
    workspace(cx).update(cx, |workspace, cx| {
        workspace.set_page(captain_ui::Page::Settings, cx)
    });
}

fn open(cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(1440.), px(900.)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        // The sidebar draws the title area; the window buttons sit inside it.
        titlebar: Some(TitlebarOptions {
            title: Some("Captain".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(18.), px(18.))),
        }),
        window_min_size: Some(size(px(1100.), px(640.))),
        app_id: Some("dev.captain.Captain".into()),
        ..Default::default()
    };

    let workspace = workspace(cx);
    let opened = gpui_kit::open_window(options, cx, |window, cx| {
        cx.new(|cx| AppShell::with_workspace(workspace, window, cx))
    });
    match opened {
        Ok((handle, _)) => cx.global_mut::<MainWindow>().handle = Some(handle),
        Err(error) => {
            tracing::error!(%error, "failed to open the main window");
            cx.quit();
        }
    }
}

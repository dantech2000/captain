//! The main window.

use captain_ui::AppShell;
use gpui_kit::*;

use crate::connect;

pub fn open(cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(1200.), px(760.)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("Captain".into()),
            ..Default::default()
        }),
        window_min_size: Some(size(px(720.), px(420.))),
        app_id: Some("dev.captain.Captain".into()),
        ..Default::default()
    };

    let opened = gpui_kit::open_window(options, cx, |window, cx| {
        cx.new(|cx| AppShell::new(connect::docker(), window, cx))
    });
    if let Err(error) = opened {
        tracing::error!(%error, "failed to open the main window");
        cx.quit();
    }
}

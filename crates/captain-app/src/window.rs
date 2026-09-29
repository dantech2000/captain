//! The main window.

use captain_ui::AppShell;
use gpui_kit::*;

use crate::connect;

pub fn open(cx: &mut App) {
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

    let opened = gpui_kit::open_window(options, cx, |window, cx| {
        cx.new(|cx| AppShell::new(connect::docker(), window, cx))
    });
    if let Err(error) = opened {
        tracing::error!(%error, "failed to open the main window");
        cx.quit();
    }
}

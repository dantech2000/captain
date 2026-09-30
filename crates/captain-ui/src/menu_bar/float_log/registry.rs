//! One floating log window per container: opening it again brings it forward.

use std::collections::HashMap;

use gpui_kit::*;

use super::float_log_view::FloatLogView;
use crate::workspace::Workspace;

/// The open log windows by container ID. A handle goes stale when the user closes
/// the window.
#[derive(Default)]
struct Windows(HashMap<String, AnyWindowHandle>);

impl Global for Windows {}

/// Opens the floating log of container `id`, or brings it to the front.
pub fn open_float_log(workspace: Entity<Workspace>, id: String, name: String, cx: &mut App) {
    let handle = cx.default_global::<Windows>().0.get(&id).copied();
    let activated = handle.is_some_and(|handle| {
        handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    });
    if activated {
        return;
    }
    let project = workspace
        .read(cx)
        .store()
        .find(&id)
        .and_then(|c| c.compose_project.clone());
    let title = match &project {
        Some(project) => format!("{name} · {project}"),
        None => name.clone(),
    };
    let bounds = Bounds::centered(None, size(px(440.), px(250.)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some(title.into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(12.), px(11.))),
        }),
        kind: WindowKind::Floating,
        is_minimizable: false,
        window_min_size: Some(size(px(300.), px(140.))),
        app_id: Some("dev.captain.Captain".into()),
        ..Default::default()
    };
    let key = id.clone();
    let opened = gpui_kit::open_window(options, cx, |window, cx| {
        cx.new(|cx| FloatLogView::new(workspace, id, name, project, window, cx))
    });
    match opened {
        Ok((handle, _)) => {
            cx.default_global::<Windows>().0.insert(key, handle);
        }
        Err(error) => tracing::error!(%error, "cannot open the floating log"),
    }
}

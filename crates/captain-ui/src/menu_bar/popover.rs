//! The one popover under the menu bar icon: a click opens it, a second click or a
//! click elsewhere closes it.

use std::time::{Duration, Instant};

use gpui_kit::*;

use super::popover_data::PopoverData;
use super::popover_view::PopoverView;
use crate::workspace::Workspace;

/// The popover's width, from the Lighthouse design.
pub const POPOVER_WIDTH: f32 = 380.;

/// A click on the icon closes the popover before the icon reports the click. A
/// click this soon after the popover closed on its own only closes it.
const REOPEN_GUARD: Duration = Duration::from_millis(500);

#[derive(Default)]
struct Popover {
    handle: Option<AnyWindowHandle>,
    closed_at: Option<Instant>,
}

impl Global for Popover {}

/// Opens the popover, or closes it when it is open. `place` gets the size the
/// content wants and returns the display and the bounds on it, or `None` when the
/// icon is on no known screen. `open_captain` brings up the main window.
pub fn toggle_popover(
    workspace: Entity<Workspace>,
    place: impl FnOnce(Size<Pixels>) -> Option<(Option<DisplayId>, Bounds<Pixels>)>,
    open_captain: fn(&mut App),
    cx: &mut App,
) {
    if close_popover(cx) {
        return;
    }
    let state = cx.default_global::<Popover>();
    if state
        .closed_at
        .is_some_and(|at| at.elapsed() < REOPEN_GUARD)
    {
        return;
    }
    let height = PopoverData::gather(workspace.read(cx), &Default::default(), cx).height();
    let Some((display_id, bounds)) = place(size(px(POPOVER_WIDTH), height)) else {
        return;
    };
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        kind: WindowKind::PopUp,
        is_movable: false,
        is_resizable: false,
        is_minimizable: false,
        display_id,
        app_id: Some("dev.captain.Captain".into()),
        ..Default::default()
    };
    let opened = gpui_kit::open_window(options, cx, |window, cx| {
        cx.new(|cx| PopoverView::new(workspace, open_captain, window, cx))
    });
    match opened {
        Ok((handle, _)) => cx.default_global::<Popover>().handle = Some(handle),
        Err(error) => tracing::error!(%error, "cannot open the menu bar popover"),
    }
}

/// Closes the popover. True if it was open.
pub fn close_popover(cx: &mut App) -> bool {
    let Some(handle) = cx.default_global::<Popover>().handle.take() else {
        return false;
    };
    handle
        .update(cx, |_, window, _| window.remove_window())
        .is_ok()
}

/// The popover lost focus or the user pressed Escape. Remembers the time, so the
/// icon click that caused it does not open the popover again.
pub(super) fn closed_itself(window: &mut Window, cx: &mut App) {
    let state = cx.default_global::<Popover>();
    state.handle = None;
    state.closed_at = Some(Instant::now());
    window.remove_window();
}

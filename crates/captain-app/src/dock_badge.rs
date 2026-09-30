//! The number on Captain's Dock icon: failed diagnostics checks, and containers that
//! are restarting or unhealthy. macOS only. See feature 0032.

use gpui_kit::*;
use objc2::MainThreadMarker;
use objc2_app_kit::NSApplication;
use objc2_foundation::NSString;

use crate::window;

/// Shows the count on the Dock icon, and follows it from then on. Call it on the
/// main thread after the app has launched.
pub fn follow(cx: &mut App) {
    let workspace = window::workspace(cx);
    captain_ui::observe_problem_count(&workspace, |count, _| show(count), cx);
}

/// Sets the badge, or clears it at 0. AppKit wants the main thread; GPUI runs its
/// callbacks there.
fn show(count: usize) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let label = (count > 0).then(|| NSString::from_str(&count.to_string()));
    NSApplication::sharedApplication(mtm)
        .dockTile()
        .setBadgeLabel(label.as_deref());
}

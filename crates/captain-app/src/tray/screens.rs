//! The screens the popover can open on, in logical pixels with the origin at the
//! top left of the primary screen.

use gpui_kit::*;

#[cfg(target_os = "macos")]
use super::placement::Rect;
use super::placement::Screen;

/// AppKit's screens, the one under the mouse first. AppKit puts the origin at the
/// bottom left of the primary screen, so the frames flip.
#[cfg(target_os = "macos")]
pub fn screens(_: &mut App) -> Vec<Screen<Option<DisplayId>>> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSEvent, NSScreen};
    use objc2_foundation::NSRect;

    let Some(mtm) = MainThreadMarker::new() else {
        return Vec::new();
    };
    let all = NSScreen::screens(mtm);
    let Some(primary) = all.iter().next() else {
        return Vec::new();
    };
    let top = primary.frame().size.height;
    let flip = |rect: NSRect| Rect {
        x: rect.origin.x as f32,
        y: (top - rect.origin.y - rect.size.height) as f32,
        width: rect.size.width as f32,
        height: rect.size.height as f32,
    };
    let mut screens: Vec<_> = all
        .iter()
        .map(|screen| Screen {
            id: Some(DisplayId::new(u64::from(screen.CGDirectDisplayID()))),
            frame: flip(screen.frame()),
            visible: flip(screen.visibleFrame()),
            scale: screen.backingScaleFactor() as f32,
        })
        .collect();
    let mouse = NSEvent::mouseLocation();
    let (x, y) = (mouse.x as f32, (top - mouse.y) as f32);
    screens.sort_by_key(|screen| !screen.frame.contains(x, y));
    screens
}

/// GPUI's displays. GPUI does not tell a display's scale, so every display gets the
/// scale of an open Captain window, or 1.
#[cfg(target_os = "windows")]
pub fn screens(cx: &mut App) -> Vec<Screen<Option<DisplayId>>> {
    use super::placement::Rect;

    let rect = |bounds: Bounds<Pixels>| Rect {
        x: bounds.origin.x.as_f32(),
        y: bounds.origin.y.as_f32(),
        width: bounds.size.width.as_f32(),
        height: bounds.size.height.as_f32(),
    };
    let scale = cx
        .windows()
        .into_iter()
        .find_map(|handle| handle.update(cx, |_, window, _| window.scale_factor()).ok())
        .unwrap_or(1.);
    cx.displays()
        .iter()
        .map(|display| Screen {
            id: Some(display.id()),
            frame: rect(display.bounds()),
            visible: rect(display.visible_bounds()),
            scale,
        })
        .collect()
}

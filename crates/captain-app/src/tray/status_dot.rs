//! The stop-light dot on macOS. A status item image is either a template, which
//! macOS tints for light and dark menu bars, or a color image, which it draws as
//! is. The wheel must follow the menu bar and the dot must keep its color, so this
//! draws both in an image with a drawing handler: AppKit calls it each time it
//! draws the image, with the menu bar's appearance current, so `labelColor` and the
//! system colors resolve for that menu bar, "Reduce transparency", and "Increase
//! contrast". See feature 0009 and
//! <https://developer.apple.com/documentation/appkit/nsimage/init(size:flipped:drawinghandler:)>.

use block2::RcBlock;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2_app_kit::{NSBezierPath, NSColor, NSCompositingOperation, NSImage, NSRectFill};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use tray_icon::TrayIcon;

use super::dot::Light;
use super::icon;

/// Replaces the status item's template image, which tray-icon just set and which
/// has a clear place for the dot, with the wheel in the menu bar's text color and
/// the dot in `light`'s system color.
pub fn paint(tray: &TrayIcon, light: Light) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let Some(button) = tray.ns_status_item().and_then(|item| item.button(mtm)) else {
        return;
    };
    let Some(wheel) = button.image() else {
        return;
    };
    let size = wheel.size();
    let dot = dot_rect(size);
    let color = system_color(light);
    let draw = RcBlock::new(move |rect: NSRect| -> Bool {
        NSColor::labelColor().set();
        NSRectFill(rect);
        // Keeps the text color only where the wheel is.
        wheel.drawInRect_fromRect_operation_fraction(
            rect,
            NSRect::ZERO,
            NSCompositingOperation::DestinationIn,
            1.,
        );
        color.set();
        NSBezierPath::bezierPathWithOvalInRect(dot).fill();
        Bool::YES
    });
    let image = NSImage::imageWithSize_flipped_drawingHandler(size, false, &draw);
    button.setImage(Some(&image));
}

/// The dot in points, in an image of `size` whose origin is at the bottom left.
fn dot_rect(size: NSSize) -> NSRect {
    let scale = size.height / f64::from(icon::SIZE);
    let (x, y, radius) = icon::DOT;
    let (x, y, radius) = (f64::from(x), f64::from(y), f64::from(radius));
    let bottom = f64::from(icon::SIZE) - y - radius;
    NSRect::new(
        NSPoint::new((x - radius) * scale, bottom * scale),
        NSSize::new(2. * radius * scale, 2. * radius * scale),
    )
}

/// Apple's system colors: they adapt to light and dark menu bars and to "Increase
/// contrast". Orange reads better than yellow on a light menu bar.
fn system_color(light: Light) -> Retained<NSColor> {
    match light {
        Light::Green => NSColor::systemGreenColor(),
        Light::Amber => NSColor::systemOrangeColor(),
        Light::Red => NSColor::systemRedColor(),
        Light::Gray => NSColor::systemGrayColor(),
    }
}

//! Where the popover opens: under the menu bar icon, or above the icon on a taskbar
//! at the bottom, kept inside the icon's screen. See feature 0032.

/// A rectangle with the origin at its top left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    fn right(&self) -> f32 {
        self.x + self.width
    }

    fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }
}

/// One screen, in logical pixels, with the origin at the top left of the primary
/// screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Screen<Id> {
    pub id: Id,
    pub frame: Rect,
    /// The frame without the menu bar, the Dock, or the taskbar.
    pub visible: Rect,
    /// Physical pixels per logical pixel.
    pub scale: f32,
}

/// Space between the icon and the popover, and between the popover and the screen
/// edge.
const GAP: f32 = 4.;
const MARGIN: f32 = 8.;

/// The screen and the popover's rectangle for a popover of `width` by `height`.
/// `icon` is the icon's rectangle in physical pixels, as tray-icon reports it.
/// `screens` come in order of preference: the first whose frame holds the icon at
/// its own scale wins. Screens with different scales can both seem to hold it, so
/// the caller puts the screen under the mouse first. When
/// `relative` is true (macOS), the rectangle is relative to the screen's top left,
/// as GPUI places windows there; otherwise it is global (Windows). `None` when the
/// icon is on no screen.
pub fn popover_rect<Id: Copy>(
    icon: Rect,
    screens: &[Screen<Id>],
    width: f32,
    height: f32,
    relative: bool,
) -> Option<(Id, Rect)> {
    let (screen, icon) = screens.iter().find_map(|screen| {
        let icon = Rect {
            x: icon.x / screen.scale,
            y: icon.y / screen.scale,
            width: icon.width / screen.scale,
            height: icon.height / screen.scale,
        };
        let center = (icon.x + icon.width / 2., icon.y + icon.height / 2.);
        screen
            .frame
            .contains(center.0, center.1)
            .then_some((screen, icon))
    })?;
    let area = screen.visible;
    let height = height.min(area.height - 2. * MARGIN);
    let left = icon.x + icon.width / 2. - width / 2.;
    let x = left.min(area.right() - MARGIN - width).max(area.x + MARGIN);
    let below = icon.bottom() + GAP;
    let y = if below + height <= area.bottom() - MARGIN {
        below
    } else {
        (icon.y - GAP - height).max(area.y + MARGIN)
    };
    let (dx, dy) = if relative {
        (screen.frame.x, screen.frame.y)
    } else {
        (0., 0.)
    };
    let rect = Rect {
        x: x - dx,
        y: y - dy,
        width,
        height,
    };
    Some((screen.id, rect))
}

#[cfg(test)]
mod tests;

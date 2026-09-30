//! The colored status dots in the menu: the engine, the problem, each container and
//! project, and Kubernetes. Drawn in code like the menu bar icon, and kept in color
//! (not template images), so they read as status lights in light and dark menus.

use muda::Icon;

use super::icon::{SIZE, coverage};

/// muda scales an item's image to 18 points high on macOS, and to 16 pixels on
/// Windows, so the dot uses the menu bar icon's canvas: 36 pixels, 18 points at 2x.
/// A 14 pixel dot shows at 7 points.
const RADIUS: f32 = 7.;
const CENTER: f32 = SIZE as f32 / 2.;

/// A status color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Light {
    /// Running.
    Green,
    /// Starting, stopping, paused, or partly running.
    Amber,
    /// Failed, crashing, or out of memory.
    Red,
    /// Stopped or off.
    Gray,
}

impl Light {
    const ALL: [Self; 4] = [Self::Green, Self::Amber, Self::Red, Self::Gray];

    /// Apple's system colors. The menu follows the system appearance, not Captain's
    /// theme, so the dots use the colors the system's own status dots use.
    fn rgb(self) -> [u8; 3] {
        match self {
            Self::Green => [0x34, 0xC7, 0x59],
            Self::Amber => [0xFF, 0x9F, 0x0A],
            Self::Red => [0xFF, 0x3B, 0x30],
            Self::Gray => [0x8E, 0x8E, 0x93],
        }
    }

    /// The dot as a menu item image.
    pub fn icon(self) -> Icon {
        DOTS.with(|dots| dots[self as usize].clone())
    }
}

thread_local! {
    /// One image per color. Menus are built on the main thread.
    static DOTS: [Icon; 4] = Light::ALL.map(|light| {
        Icon::from_rgba(rgba(light), SIZE, SIZE).expect("the dot buffer matches its size")
    });
}

/// The dot in `light` as RGBA rows, `SIZE` by `SIZE`, with anti-aliased edges.
fn rgba(light: Light) -> Vec<u8> {
    let color = light.rgb();
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let alpha = coverage(x, y, |px, py| (px - CENTER).hypot(py - CENTER) <= RADIUS);
            pixels.extend_from_slice(&color);
            pixels.push((alpha * 255.).round() as u8);
        }
    }
    pixels
}

#[cfg(test)]
mod tests;

//! The colored status lights in the menu: the engine, the problem, each container and
//! project, and Kubernetes.
//!
//! They are colored circle characters at the start of the item's text. The system
//! menu of a status item shows no item images on current macOS (neither drawn nor
//! `NSImageNameStatusAvailable`), and muda styles text only as secondary gray, but
//! emoji always draw in color, in light and dark menus, on macOS and Windows.

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
    /// The circle for this color.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Green => "\u{1F7E2}",
            Self::Amber => "\u{1F7E1}",
            Self::Red => "\u{1F534}",
            Self::Gray => "\u{26AA}\u{FE0F}",
        }
    }

    /// The sRGB color of the icon's dot where the system has no dynamic colors
    /// (Windows): Apple's dark-mode system green, orange, red, and gray, to match
    /// the white wheel on the dark taskbar. macOS uses `NSColor`'s system colors.
    /// See <https://developer.apple.com/design/human-interface-guidelines/color>.
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Self::Green => [48, 209, 88],
            Self::Amber => [255, 159, 10],
            Self::Red => [255, 69, 58],
            Self::Gray => [152, 152, 157],
        }
    }

    /// `label` with this light in front.
    pub fn label(self, label: &str) -> String {
        format!("{} {label}", self.glyph())
    }
}

#[cfg(test)]
mod tests;

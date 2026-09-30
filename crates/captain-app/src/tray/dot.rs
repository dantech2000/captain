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

    /// `label` with this light in front.
    pub fn label(self, label: &str) -> String {
        format!("{} {label}", self.glyph())
    }
}

#[cfg(test)]
mod tests;

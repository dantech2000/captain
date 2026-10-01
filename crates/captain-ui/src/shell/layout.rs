//! The widths of the window's left side, which the inspector's resize needs.

/// The icon rail's width.
pub(super) const RAIL_WIDTH: f32 = 56.;
/// The projects list's width, while it shows.
pub(super) const SIDEBAR_WIDTH: f32 = 256.;

/// What the rail and the projects list take from the window's left side.
pub(crate) fn left_width(sidebar_hidden: bool) -> f32 {
    if sidebar_hidden {
        RAIL_WIDTH
    } else {
        RAIL_WIDTH + SIDEBAR_WIDTH
    }
}

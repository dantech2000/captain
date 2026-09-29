//! Small building blocks shared by the views.

mod drag_region;
mod icon_button;
mod pill;
pub mod scales;
mod segmented;
mod sparkline;
mod stat_tile;
mod status_dot;

pub use drag_region::drag_region;
pub use icon_button::icon_button;
pub use pill::pill;
pub use segmented::{Segment, segmented};
pub use sparkline::{Scale, sparkline};
pub use stat_tile::{StatTile, stat_tile};
pub use status_dot::status_dot;

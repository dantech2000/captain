//! The floating log window and the problem count for the Dock badge. The menu bar
//! icon and its native menu are in `captain-app`. See
//! docs/features/0032-menu-bar-popover.md.

mod float_log;
mod problem_count;

pub use float_log::open_float_log;
pub use problem_count::observe_problem_count;

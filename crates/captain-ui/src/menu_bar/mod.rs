//! The menu bar popover, the floating log window, and the problem count for the
//! Dock badge. The tray icon itself is in `captain-app`. See
//! docs/features/0032-menu-bar-popover.md.

mod controls;
mod engine_header;
mod float_log;
mod footer;
mod popover;
mod popover_data;
mod popover_view;
mod ports_section;
mod problem_count;
mod projects_section;
mod warning_card;

pub use float_log::open_float_log;
pub use popover::{close_popover, toggle_popover};
pub use problem_count::observe_problem_count;

//! The terminal panel: shells on this computer in tabs under the page, with
//! `docker` pointed at Captain's engine. See docs/features/0041-integrated-terminal.md.

mod keys;
mod new_tab;
mod panel_view;
mod resize_edge;
mod tab_strip;
mod tab_title;

pub use keys::{NewTerminalTab, TOGGLE_KEYS, ToggleTerminal, terminal_bindings};
pub use new_tab::default_dir;
pub use panel_view::TerminalPanel;
pub use resize_edge::DEFAULT_HEIGHT;

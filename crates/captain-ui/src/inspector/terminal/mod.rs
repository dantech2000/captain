//! The Terminal tab: an interactive shell in the selected container. See
//! docs/features/0011-terminal.md.

mod header;
mod status;
mod terminal_pane;

pub use terminal_pane::{TerminalPane, TerminalTarget};

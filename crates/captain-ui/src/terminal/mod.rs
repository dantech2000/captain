//! The terminal view that the inspector's Terminal tab and the terminal panel share:
//! the grid, keys, selection, and a session from a [`TerminalSource`]. See
//! docs/features/0011-terminal.md and docs/features/0041-integrated-terminal.md.

mod closing;
mod colors;
mod events;
mod exec_source;
mod grid;
mod input;
mod keys;
mod local_source;
mod metrics;
mod output_budget;
mod overlay;
mod pty_session;
mod session;
mod source;
mod terminal_view;

pub use closing::Closing;
pub use exec_source::ExecSource;
pub use local_source::LocalSource;
pub use source::TerminalSource;
pub use terminal_view::{CloseRequested, EndedBar, Phase, TerminalView};

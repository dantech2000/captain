//! Captain Engine in the UI: the host model, the screens the Containers page shows
//! while the engine is not running, and the screen every engine page shows while
//! the connection is down. See docs/adr/0008-captain-engine.md.

mod connection_screen;
mod host_actions;
mod host_daemon;
mod host_event;
mod host_files;
mod host_kubernetes;
mod host_model;
mod host_reload;
mod host_restart;
mod host_screen;
mod host_snapshot;
mod host_summary;
mod other_engines;
mod progress_log;
mod setup_screen;
mod starting_screen;
mod stopped_screen;

pub use connection_screen::render as connection_screen;
pub use host_event::HostEvent;
pub use host_model::{HostModel, captain_endpoint, captain_socket, host_model, init, uses_captain};
pub use host_screen::render as host_screen;
pub use host_summary::{HostSummary, summary};
pub use progress_log::ProgressLog;

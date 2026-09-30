//! Captain Engine in the UI: the host model, and the screens the Containers page
//! shows while the engine is not running. See docs/adr/0008-captain-engine.md.

mod host_actions;
mod host_daemon;
mod host_event;
mod host_files;
mod host_kubernetes;
mod host_model;
mod host_reload;
mod host_screen;
mod host_snapshot;
mod host_summary;
mod progress_log;
mod setup_screen;
mod starting_screen;
mod stopped_screen;

pub use host_event::HostEvent;
pub use host_model::{HostModel, captain_endpoint, captain_socket, host_model, init, uses_captain};
pub use host_screen::render as host_screen;
pub use host_summary::{HostSummary, summary};
pub use progress_log::ProgressLog;

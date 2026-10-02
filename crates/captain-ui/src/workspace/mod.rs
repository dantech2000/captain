//! The shared state the views observe: the engine connection, the container list,
//! live stats, the list filter, the selected container, folded project cards,
//! Compose project actions, and the sidebar entry the Project page shows.

mod actions;
mod auto_reconnect;
mod bulk;
mod connect;
mod containers;
mod engine_health;
mod focus;
mod page;
mod projects;
mod reveal;
mod stats_feed;
mod terminal;
mod workspace_event;
mod workspace_state;

pub use auto_reconnect::{expected_running, resume_endpoint};
pub use connect::{Connector, active_workspace};
pub use engine_health::EngineHealth;
pub use focus::{InspectorTab, LogFilter};
pub use page::Page;
pub use workspace_event::WorkspaceEvent;
pub use workspace_state::{Connection, Workspace};

//! The shared state the views observe: the engine connection, the container list,
//! live stats, the list filter, the selected container, folded project cards,
//! Compose project actions, and the sidebar entry the Project page shows.

mod actions;
mod bulk;
mod connect;
mod containers;
mod focus;
mod page;
mod projects;
mod reveal;
mod stats_feed;
mod workspace_event;
mod workspace_state;

pub use connect::{Connector, active_workspace};
pub use focus::{InspectorTab, LogFilter};
pub use page::Page;
pub use workspace_event::WorkspaceEvent;
pub use workspace_state::{Connection, Workspace};

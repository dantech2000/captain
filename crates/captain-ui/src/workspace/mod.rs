//! The shared state the views observe: the engine connection, the container list,
//! live stats, the list filter, the selected container, folded project cards, and
//! Compose project actions.

mod actions;
mod bulk;
mod connect;
mod containers;
mod page;
mod projects;
mod stats_feed;
mod workspace_event;
mod workspace_state;

pub use connect::{Connector, active_workspace};
pub use page::Page;
pub use workspace_event::WorkspaceEvent;
pub use workspace_state::{Connection, Workspace};

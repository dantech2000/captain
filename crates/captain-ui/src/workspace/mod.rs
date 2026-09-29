//! The shared state the views observe: the engine connection, the container list,
//! live stats, the list filter, and the selected container.

mod actions;
mod connect;
mod containers;
mod stats_feed;
mod workspace_state;

pub use connect::Connector;
pub use workspace_state::{Connection, Workspace};

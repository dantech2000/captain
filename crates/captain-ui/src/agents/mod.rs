//! AI agents: the sheet that connects them to Captain's MCP server and sets what
//! they may do, and the watch on their activity. See
//! docs/features/0038-agent-tools.md.

mod activity_list;
mod activity_watch;
mod agents_sheet;
mod client_rows;
mod clients_state;

pub use activity_list::time as activity_time;
pub use activity_watch::{agent_activity, init, latest_activity};
pub use agents_sheet::open as open_agents_sheet;

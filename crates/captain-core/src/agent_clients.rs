//! Connects AI agents to Captain's MCP server: finds Claude Code, Codex, Gemini
//! CLI, VS Code, Cursor, Zed, and Claude Desktop, and adds or removes the `captain`
//! server with each client's own installer, a link it confirms, or an in-place edit
//! of its file. See docs/features/0038-agent-tools.md.

mod client;
mod config_edit;
mod detect;
mod paths;
mod plan;
mod run;

pub use client::{AgentClient, ConfigFormat, SERVER_NAME, client_label};
pub use config_edit::{codex_has_server, has_server, with_server, without_server};
pub use detect::{ClientState, detect, find_commands};
pub use paths::{ClientPaths, captain_command};
pub use plan::{ClientStep, connect_step, copy_config, remove_step, server_entry};
pub use run::{line_diff, login_shell_command, run_step, shell_line};

//! The command-line tools in the user's terminal: links in `~/.captain/bin` into
//! `Captain.app`, Captain's plugin folder in the user's docker `config.json`, and
//! the PATH lines in the shell files. See docs/features/0035-command-line-tools.md.

mod chezmoi;
mod command;
mod install;
mod links;
mod paths;
mod plugin_config;
mod rc_block;
mod rc_files;
mod resolve;
mod settings;

pub use install::{RcStatus, ToolsStatus, install, status, uninstall};
pub use links::{LinkReport, LinkState, ToolLink, link_state, relink, remove_links, tool_links};
pub use paths::{SUPPORTED, ToolPaths, UNSUPPORTED, login_shell, running_bundle};
pub use plugin_config::{
    add_plugin_dir, has_plugin_dir, remove_plugin_dir, with_plugin_dir, without_plugin_dir,
};
pub use rc_block::{END, START, with_block, without_block};
pub use rc_files::{
    RcAccess, RcFile, RcState, Shell, add_block, rc_access, rc_files, rc_state, remove_block,
};
pub use resolve::{SHOWN_TOOLS, ToolSource, classify, parse_command_v, resolve_in_login_shell};
pub use settings::{CliToolsSettings, PathMode};

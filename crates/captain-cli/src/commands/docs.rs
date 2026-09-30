//! `captain docs cli` and `captain docs mcp`: the Markdown references of the
//! commands and of the MCP tools.

use crate::cli::DocsCommand;
use crate::{mcp, reference};

pub fn run(command: DocsCommand) {
    match command {
        DocsCommand::Cli => print!("{}", reference::markdown()),
        DocsCommand::Mcp => print!("{}", mcp::reference_markdown()),
    }
}

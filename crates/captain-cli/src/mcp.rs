//! Captain's MCP server for AI agents, over stdio. See
//! docs/features/0038-agent-tools.md.

mod action_tools;
mod detail_tools;
mod handler;
mod log_tools;
mod overview_tools;
mod params;
mod reference;
mod reply;
mod server;
mod source;

pub use reference::markdown as reference_markdown;
pub use server::CaptainServer;
pub use source::{Connect, Connected, ReadSettings, Source};

#[cfg(test)]
mod tests;

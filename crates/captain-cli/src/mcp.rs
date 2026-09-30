//! Captain's MCP server for AI agents, over stdio. See
//! docs/features/0038-agent-tools.md.

mod detail_tools;
mod log_tools;
mod overview_tools;
mod params;
mod reply;
mod server;
mod source;

pub use server::CaptainServer;
pub use source::{Connect, Source};

#[cfg(test)]
mod tests;

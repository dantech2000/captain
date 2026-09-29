//! Compose projects through the `docker compose` CLI, because the Engine API has no
//! Compose endpoints. See docs/adr/0005-compose-via-cli.md.

mod cli;
mod command;
mod locate;
mod output;

pub use cli::ComposeCli;

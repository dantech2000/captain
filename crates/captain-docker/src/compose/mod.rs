//! Compose projects through the `docker compose` CLI, because the Engine API has no
//! Compose endpoints. See docs/adr/0005-compose-via-cli.md.

mod cli;
mod command;
mod docker_cli;
mod locate;
mod output;
mod tools;

pub use cli::ComposeCli;
pub use tools::docker_tools;

pub(crate) use command::docker_host;
pub(crate) use docker_cli::DockerCli;
pub(crate) use locate::locate_helper;
pub(crate) use output::error_message;

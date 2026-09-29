//! Docker Desktop extensions on the engine: install, remove, and the engine side of
//! the `ddClient` bridge. See docs/adr/0011-extensions.md.

mod backend;
mod bridge;
mod exec_policy;
mod files;
mod install;
mod manager;
mod process;
mod swap;
mod update;

pub use manager::DockerExtensions;

//! The Docker Engine API implementation of [`captain_core::Engine`], built on bollard.
//! Bollard types stay inside this crate.

mod build;
mod child;
mod compose;
mod credentials;
mod discovery;
mod docker_contexts;
mod endpoint;
mod engine;
mod extensions;
mod hub;
mod mapping;
mod process;
mod process_group;
mod runtime;
mod ssh_tunnel;
mod transfer;

pub use build::BuildCli;
pub use compose::{ComposeCli, docker_tools};
pub use discovery::{
    Candidate, CandidateSource, DiscoveryError, DiscoveryInput, candidates, discover, discover_host,
};
pub use docker_contexts::{save_context, use_context};
pub use endpoint::{Endpoint, UnsupportedHost};
pub use engine::DockerEngine;
pub use extensions::DockerExtensions;
pub use hub::DockerHubClient;
pub use ssh_tunnel::{close_ssh_tunnel, open_ssh_tunnel};
pub use transfer::{DockerMigrator, DockerSession};

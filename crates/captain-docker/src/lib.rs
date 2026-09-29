//! The Docker Engine API implementation of [`captain_core::Engine`], built on bollard.
//! Bollard types stay inside this crate.

mod build;
mod compose;
mod credentials;
mod discovery;
mod docker_context;
mod endpoint;
mod engine;
mod extensions;
mod mapping;
mod process;
mod runtime;
mod transfer;

pub use build::BuildCli;
pub use compose::{ComposeCli, docker_tools};
pub use discovery::{
    Candidate, CandidateSource, DiscoveryError, DiscoveryInput, candidates, discover,
};
pub use endpoint::{Endpoint, UnsupportedHost};
pub use engine::DockerEngine;
pub use extensions::DockerExtensions;
pub use transfer::{DockerMigrator, DockerSession};

//! The Docker Engine API implementation of [`captain_core::Engine`], built on bollard.
//! Bollard types stay inside this crate.

mod compose;
mod discovery;
mod docker_context;
mod endpoint;
mod engine;
mod mapping;
mod runtime;
mod transfer;

pub use compose::ComposeCli;
pub use discovery::{
    Candidate, CandidateSource, DiscoveryError, DiscoveryInput, candidates, discover,
};
pub use endpoint::{Endpoint, UnsupportedHost};
pub use engine::DockerEngine;
pub use transfer::{DockerMigrator, DockerSession};

//! The Docker Engine API implementation of [`captain_core::Engine`], built on bollard.
//! Bollard types stay inside this crate.

mod discovery;
mod docker_context;
mod endpoint;
mod engine;
mod mapping;
mod runtime;

pub use discovery::{DiscoveryError, DiscoveryInput, discover};
pub use endpoint::{Endpoint, UnsupportedHost};
pub use engine::DockerEngine;

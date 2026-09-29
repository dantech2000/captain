//! Domain models and state for Captain. This crate has no UI or Docker dependencies.

mod engine;
mod error;
mod fake_engine;
pub mod format;
pub mod migration;
pub mod model;
pub mod search;
pub mod settings;
pub mod store;

pub use engine::{
    ContainerApi, Engine, EngineFuture, EngineHost, EngineStream, GIB, HostError, HostFuture,
    HostResources, HostStatus, HostStream, ImageApi, NetworkApi, ProjectRunner, VolumeApi,
};
pub use error::EngineError;
pub use fake_engine::{FakeEngine, FakeImages, FakeNetworks, FakeVolumes};

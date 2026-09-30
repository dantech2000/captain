//! Domain models and state for Captain. This crate has no UI or Docker dependencies.

pub mod agent_tools;
pub mod behavior;
pub mod cli_tools;
pub mod daemon;
pub mod diagnostics;
pub mod docker_context;
mod engine;
mod error;
pub mod extension;
mod fake_engine;
mod file_replace;
pub mod format;
pub mod grammar;
pub mod kubernetes;
pub mod link_target;
pub mod migration;
pub mod model;
pub mod problems;
pub mod process_lock;
pub mod project_map;
pub mod registry;
pub mod search;
pub mod settings;
pub mod snapshot;
pub mod ssh;
pub mod storage;
pub mod store;
pub mod tools;

pub use engine::{
    ContainerApi, Engine, EngineFuture, EngineHost, EngineStream, GIB, HostError, HostFuture,
    HostResources, HostStatus, HostStream, ImageApi, ImageBuilder, NetworkApi, ProjectRunner,
    VolumeApi,
};
pub use error::EngineError;
pub use fake_engine::{FakeEngine, FakeFiles, FakeImages, FakeNetworks, FakeVolumes};

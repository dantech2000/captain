//! Domain models and state for Captain. This crate has no UI or Docker dependencies.

mod engine;
mod error;
mod fake_engine;
pub mod format;
pub mod model;
pub mod store;

pub use engine::{Engine, EngineFuture, EngineStream};
pub use error::EngineError;
pub use fake_engine::FakeEngine;

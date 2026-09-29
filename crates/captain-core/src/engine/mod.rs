//! The engine API, one trait per resource. [`Engine`] is all of them together.

mod containers;
mod images;
mod networks;
mod projects;
mod volumes;

use futures::future::BoxFuture;
use futures::stream::BoxStream;

pub use containers::ContainerApi;
pub use images::ImageApi;
pub use networks::NetworkApi;
pub use projects::ProjectRunner;
pub use volumes::VolumeApi;

use crate::EngineError;

/// A future that any executor can await.
pub type EngineFuture<T> = BoxFuture<'static, Result<T, EngineError>>;
/// A stream that any executor can poll.
pub type EngineStream<T> = BoxStream<'static, Result<T, EngineError>>;

/// A container engine. The futures and streams must not depend on a specific async
/// runtime, so GPUI tasks can await them. See docs/adr/0002-bollard-and-tokio-bridge.md.
///
/// Call its methods with the resource trait in scope, for example
/// `use captain_core::ContainerApi;`.
pub trait Engine: ContainerApi + ImageApi + VolumeApi + NetworkApi + Send + Sync + 'static {}

impl<T> Engine for T where
    T: ContainerApi + ImageApi + VolumeApi + NetworkApi + Send + Sync + 'static
{
}

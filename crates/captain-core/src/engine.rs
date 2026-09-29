use futures::future::BoxFuture;
use futures::stream::BoxStream;

use crate::EngineError;
use crate::model::{Container, EngineEvent, EngineInfo};

/// A container engine. The futures and streams must not depend on a specific async
/// runtime, so GPUI tasks can await them. See docs/adr/0002-bollard-and-tokio-bridge.md.
pub trait Engine: Send + Sync + 'static {
    /// The engine version and platform.
    fn info(&self) -> BoxFuture<'static, Result<EngineInfo, EngineError>>;

    /// All containers, including stopped ones.
    fn list_containers(&self) -> BoxFuture<'static, Result<Vec<Container>, EngineError>>;

    /// The engine event stream. It ends when the connection drops.
    fn events(&self) -> BoxStream<'static, Result<EngineEvent, EngineError>>;
}

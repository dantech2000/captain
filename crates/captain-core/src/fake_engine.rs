use futures::FutureExt;
use futures::StreamExt;
use futures::future::{BoxFuture, ready};
use futures::stream::{self, BoxStream};

use crate::model::{Container, EngineEvent, EngineInfo};
use crate::{Engine, EngineError};

/// An in-memory [`Engine`] for tests and UI previews.
#[derive(Debug, Clone, Default)]
pub struct FakeEngine {
    pub info: Option<EngineInfo>,
    pub containers: Vec<Container>,
    pub events: Vec<EngineEvent>,
}

impl Engine for FakeEngine {
    fn info(&self) -> BoxFuture<'static, Result<EngineInfo, EngineError>> {
        let result = self
            .info
            .clone()
            .ok_or_else(|| EngineError::Unreachable("fake engine is offline".into()));
        ready(result).boxed()
    }

    fn list_containers(&self) -> BoxFuture<'static, Result<Vec<Container>, EngineError>> {
        ready(Ok(self.containers.clone())).boxed()
    }

    fn events(&self) -> BoxStream<'static, Result<EngineEvent, EngineError>> {
        stream::iter(self.events.clone().into_iter().map(Ok)).boxed()
    }
}

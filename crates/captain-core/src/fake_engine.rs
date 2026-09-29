use futures::FutureExt;
use futures::StreamExt;
use futures::future::ready;
use futures::stream;

use crate::engine::{EngineFuture, EngineStream};
use crate::model::{
    Container, ContainerAction, ContainerDetail, EngineEvent, EngineInfo, LogLine, StatsSample,
};
use crate::{Engine, EngineError};

/// An in-memory [`Engine`] for tests and UI previews.
#[derive(Debug, Clone, Default)]
pub struct FakeEngine {
    pub info: Option<EngineInfo>,
    pub containers: Vec<Container>,
    pub events: Vec<EngineEvent>,
    pub stats: Vec<StatsSample>,
    pub logs: Vec<LogLine>,
}

impl Engine for FakeEngine {
    fn info(&self) -> EngineFuture<EngineInfo> {
        let result = self
            .info
            .clone()
            .ok_or_else(|| EngineError::Unreachable("fake engine is offline".into()));
        ready(result).boxed()
    }

    fn list_containers(&self) -> EngineFuture<Vec<Container>> {
        ready(Ok(self.containers.clone())).boxed()
    }

    fn events(&self) -> EngineStream<EngineEvent> {
        stream::iter(self.events.clone().into_iter().map(Ok)).boxed()
    }

    fn inspect_container(&self, id: &str) -> EngineFuture<ContainerDetail> {
        let detail = ContainerDetail {
            id: id.to_string(),
            ..ContainerDetail::default()
        };
        ready(Ok(detail)).boxed()
    }

    fn stats(&self, _id: &str) -> EngineStream<StatsSample> {
        stream::iter(self.stats.clone().into_iter().map(Ok)).boxed()
    }

    fn logs(&self, _id: &str, tail: usize) -> EngineStream<LogLine> {
        let skip = self.logs.len().saturating_sub(tail);
        stream::iter(self.logs.clone().into_iter().skip(skip).map(Ok)).boxed()
    }

    fn run_action(&self, _id: &str, _action: ContainerAction) -> EngineFuture<()> {
        ready(Ok(())).boxed()
    }
}

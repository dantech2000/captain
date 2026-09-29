use futures::FutureExt;
use futures::StreamExt;
use futures::future::ready;
use futures::stream;

mod echo;

use super::FakeEngine;
use crate::model::{
    Container, ContainerAction, ContainerDetail, ContainerState, EngineEvent, EngineInfo,
    ExecSession, ExecSpec, LogLine, StatsSample,
};
use crate::{ContainerApi, EngineError, EngineFuture, EngineStream};

impl ContainerApi for FakeEngine {
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

    fn run_action(&self, id: &str, action: ContainerAction) -> EngineFuture<()> {
        let result = match self.containers.iter().find(|c| c.id == id) {
            Some(container) => check_action(container.state, action),
            None => Err(EngineError::Api(format!("No such container: {id}"))),
        };
        ready(result).boxed()
    }

    /// An echo session. Like the Docker engine, it refuses a container that is not
    /// running.
    fn exec(&self, id: &str, spec: ExecSpec) -> EngineFuture<ExecSession> {
        let result = match self.containers.iter().find(|c| c.id == id) {
            Some(container) if container.state == ContainerState::Running => {
                Ok(echo::echo_session(spec))
            }
            Some(_) => Err(EngineError::Api(format!("container {id} is not running"))),
            None => Err(EngineError::Api(format!("No such container: {id}"))),
        };
        ready(result).boxed()
    }
}

/// Refuses the actions the Docker engine refuses in `state`. The fake keeps no state,
/// so an allowed action changes nothing.
fn check_action(state: ContainerState, action: ContainerAction) -> Result<(), EngineError> {
    let refused = match action {
        ContainerAction::Pause => state != ContainerState::Running,
        ContainerAction::Unpause => state != ContainerState::Paused,
        ContainerAction::Remove => state.is_active(),
        _ => false,
    };
    if refused {
        Err(EngineError::Api(format!(
            "cannot {} a {} container",
            action.label().to_lowercase(),
            state.label()
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;

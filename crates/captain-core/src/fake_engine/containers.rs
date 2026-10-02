use std::path::{Path, PathBuf};

use futures::FutureExt;
use futures::StreamExt;
use futures::future::ready;
use futures::stream;

mod echo;
mod files;

pub use files::FakeFiles;

use super::FakeEngine;
use crate::model::{
    Container, ContainerAction, ContainerDetail, ContainerState, DiskUsage, EngineEvent,
    EngineInfo, ExecSession, ExecSpec, FileEntry, FilePreview, LogLine, LogOptions, ProcessTable,
    ResourceUpdate, StatsSample,
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
        let result = self.running(id).map(|()| echo::echo_session(spec));
        ready(result).boxed()
    }

    fn list_files(&self, id: &str, path: &str) -> EngineFuture<Vec<FileEntry>> {
        let result = self.running(id).and_then(|()| self.files.list(path));
        ready(result).boxed()
    }

    fn read_file(&self, _id: &str, path: &str, limit: u64) -> EngineFuture<FilePreview> {
        ready(self.files.read(path, limit)).boxed()
    }

    fn save_path(&self, _id: &str, path: &str, dir: &Path) -> EngineFuture<PathBuf> {
        ready(self.files.save(path, dir)).boxed()
    }

    fn top(&self, id: &str) -> EngineFuture<ProcessTable> {
        let result = self.running(id).map(|()| self.processes.clone());
        ready(result).boxed()
    }

    /// The fake's lines have no times to filter by, so `since` keeps them all.
    fn logs_with(&self, id: &str, options: LogOptions) -> EngineStream<LogLine> {
        self.logs(id, options.tail.unwrap_or(usize::MAX))
    }

    fn update_memory(&self, id: &str, _bytes: u64) -> EngineFuture<()> {
        let result = match self.containers.iter().find(|c| c.id == id) {
            Some(_) => Ok(()),
            None => Err(EngineError::Api(format!("No such container: {id}"))),
        };
        ready(result).boxed()
    }

    fn disk_usage(&self) -> EngineFuture<DiskUsage> {
        ready(Ok(self.disk.clone())).boxed()
    }

    fn update_resources(&self, id: &str, _update: ResourceUpdate) -> EngineFuture<()> {
        self.update_memory(id, 0)
    }
}

impl FakeEngine {
    /// Refuses a container that is missing or not running, like exec in the engine.
    fn running(&self, id: &str) -> Result<(), EngineError> {
        match self.containers.iter().find(|c| c.id == id) {
            Some(container) if container.state == ContainerState::Running => Ok(()),
            Some(_) => Err(EngineError::Api(format!("container {id} is not running"))),
            None => Err(EngineError::Api(format!("No such container: {id}"))),
        }
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

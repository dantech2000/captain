//! Containers, events, and engine info. Every call runs on the private tokio runtime.

mod exec;
mod files;

use std::path::{Path, PathBuf};

use std::pin::pin;

use bollard::query_parameters::{
    ListContainersOptionsBuilder, LogsOptionsBuilder, RemoveContainerOptionsBuilder,
    StatsOptionsBuilder, TopOptions,
};
use captain_core::model::{
    Container, ContainerAction, ContainerDetail, EngineEvent, EngineInfo, ExecSession, ExecSpec,
    FileEntry, FilePreview, LogLine, ProcessTable, StatsSample,
};
use captain_core::{ContainerApi, EngineFuture, EngineStream};
use futures::StreamExt;

use super::DockerEngine;
use crate::{mapping, runtime};

impl ContainerApi for DockerEngine {
    fn info(&self) -> EngineFuture<EngineInfo> {
        let docker = self.docker.clone();
        let endpoint = self.endpoint.clone();
        runtime::spawn(self.runtime.handle(), async move {
            let (version, info) = futures::try_join!(docker.version(), docker.info())
                .map_err(mapping::engine_error)?;
            Ok(mapping::engine_info(version, info, &endpoint))
        })
    }

    fn list_containers(&self) -> EngineFuture<Vec<Container>> {
        let docker = self.docker.clone();
        runtime::spawn(self.runtime.handle(), async move {
            let options = ListContainersOptionsBuilder::default().all(true).build();
            let summaries = docker
                .list_containers(Some(options))
                .await
                .map_err(mapping::engine_error)?;
            Ok(summaries.into_iter().map(mapping::container).collect())
        })
    }

    fn events(&self) -> EngineStream<EngineEvent> {
        let docker = self.docker.clone();
        runtime::forward(self.runtime.handle(), move |tx| async move {
            let mut events = pin!(docker.events(None));
            while let Some(item) = events.next().await {
                let item = item.map(mapping::event).map_err(mapping::engine_error);
                if tx.unbounded_send(item).is_err() {
                    break;
                }
            }
        })
    }

    fn inspect_container(&self, id: &str) -> EngineFuture<ContainerDetail> {
        let docker = self.docker.clone();
        let id = id.to_string();
        runtime::spawn(self.runtime.handle(), async move {
            let response = docker
                .inspect_container(&id, None)
                .await
                .map_err(mapping::engine_error)?;
            Ok(mapping::detail(response))
        })
    }

    fn stats(&self, id: &str) -> EngineStream<StatsSample> {
        let docker = self.docker.clone();
        let id = id.to_string();
        runtime::forward(self.runtime.handle(), move |tx| async move {
            let options = StatsOptionsBuilder::default().stream(true).build();
            let mut stats = pin!(docker.stats(&id, Some(options)));
            while let Some(item) = stats.next().await {
                let item = item.map(mapping::stats).map_err(mapping::engine_error);
                if tx.unbounded_send(item).is_err() {
                    break;
                }
            }
        })
    }

    fn logs(&self, id: &str, tail: usize) -> EngineStream<LogLine> {
        let docker = self.docker.clone();
        let id = id.to_string();
        runtime::forward(self.runtime.handle(), move |tx| async move {
            let options = LogsOptionsBuilder::default()
                .follow(true)
                .stdout(true)
                .stderr(true)
                .timestamps(true)
                .tail(&tail.to_string())
                .build();
            let mut output = pin!(docker.logs(&id, Some(options)));
            while let Some(item) = output.next().await {
                let lines = match item {
                    Ok(frame) => mapping::log_lines(frame).into_iter().map(Ok).collect(),
                    Err(error) => vec![Err(mapping::engine_error(error))],
                };
                if lines
                    .into_iter()
                    .any(|line| tx.unbounded_send(line).is_err())
                {
                    break;
                }
            }
        })
    }

    fn run_action(&self, id: &str, action: ContainerAction) -> EngineFuture<()> {
        let docker = self.docker.clone();
        let id = id.to_string();
        runtime::spawn(self.runtime.handle(), async move {
            let result = match action {
                ContainerAction::Start => docker.start_container(&id, None).await,
                ContainerAction::Stop => docker.stop_container(&id, None).await,
                ContainerAction::Restart => docker.restart_container(&id, None).await,
                ContainerAction::Pause => docker.pause_container(&id).await,
                ContainerAction::Unpause => docker.unpause_container(&id).await,
                ContainerAction::Remove => docker.remove_container(&id, None).await,
                ContainerAction::ForceRemove => {
                    let options = RemoveContainerOptionsBuilder::default().force(true).build();
                    docker.remove_container(&id, Some(options)).await
                }
            };
            result.map_err(mapping::engine_error)
        })
    }

    fn exec(&self, id: &str, spec: ExecSpec) -> EngineFuture<ExecSession> {
        let handle = self.runtime.handle().clone();
        exec::start(self.docker.clone(), handle, id.to_string(), spec)
    }

    fn list_files(&self, id: &str, path: &str) -> EngineFuture<Vec<FileEntry>> {
        let docker = self.docker.clone();
        let (id, path) = (id.to_string(), path.to_string());
        runtime::spawn(self.runtime.handle(), async move {
            files::list(&docker, &id, &path).await
        })
    }

    fn read_file(&self, id: &str, path: &str, limit: u64) -> EngineFuture<FilePreview> {
        let docker = self.docker.clone();
        let (id, path) = (id.to_string(), path.to_string());
        runtime::spawn(self.runtime.handle(), async move {
            files::read(&docker, &id, &path, limit).await
        })
    }

    fn save_path(&self, id: &str, path: &str, dir: &Path) -> EngineFuture<PathBuf> {
        let docker = self.docker.clone();
        let (id, path, dir) = (id.to_string(), path.to_string(), dir.to_path_buf());
        runtime::spawn(self.runtime.handle(), async move {
            files::save(&docker, &id, &path, dir).await
        })
    }

    fn top(&self, id: &str) -> EngineFuture<ProcessTable> {
        let docker = self.docker.clone();
        let id = id.to_string();
        runtime::spawn(self.runtime.handle(), async move {
            let response = docker
                .top_processes(&id, None::<TopOptions>)
                .await
                .map_err(mapping::engine_error)?;
            Ok(mapping::processes(response))
        })
    }
}

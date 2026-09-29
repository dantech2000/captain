//! A migration session between two engines. Every call runs on the session's own
//! tokio runtime and comes back through runtime-neutral futures and streams.

use std::sync::Arc;
use std::time::Duration;

use bollard::Docker;
use captain_core::migration::{MigrationItem, MigrationPlan, MigrationSession, TransferEvent};
use captain_core::{EngineError, EngineFuture, EngineStream};
use tokio::runtime::Runtime;

use super::compose::{Project, copy_project};
use super::container::copy_container;
use super::image::copy_image;
use super::in_flight::InFlight;
use super::network::copy_network;
use super::progress::{Events, Outcome};
use super::source::SourceEngine;
use super::switch_over::{Job, roll_back, switch_over};
use super::volume::{Existing, copy_volume};
use super::{client, disk, helper, scan};
use crate::{ComposeCli, Endpoint, runtime};

/// Copies from a source engine into a target engine. The source is reachable only
/// through [`SourceEngine`], which cannot change it.
pub struct DockerSession {
    runtime: Runtime,
    source: Arc<SourceEngine>,
    target: Docker,
    source_host: String,
    target_endpoint: Endpoint,
    in_flight: InFlight,
    /// Switch-overs that run. The session's end waits for these without a limit,
    /// because a switch-over runs to the end once it stopped the source.
    switching: InFlight,
}

/// How long [`MigrationSession::finish`] waits for stopped copies to clean up.
const CLEANUP_WAIT: Duration = Duration::from_secs(60);

impl DockerSession {
    /// Connects to both engines. It blocks, so call it from a plain thread.
    pub fn connect(source: &Endpoint, target: &Endpoint) -> Result<Self, EngineError> {
        let runtime = runtime::build().map_err(|err| EngineError::Unreachable(err.to_string()))?;
        let source_docker = client::connect(runtime.handle(), source)?;
        let target_docker = client::connect(runtime.handle(), target)?;
        Ok(Self {
            runtime,
            source: Arc::new(SourceEngine::new(source_docker)),
            target: target_docker,
            source_host: source.to_string(),
            target_endpoint: target.clone(),
            in_flight: InFlight::default(),
            switching: InFlight::default(),
        })
    }

    /// True if both endpoints reach the same engine, for example through a link.
    pub fn same_engine(&self) -> bool {
        let source = self.source.clone();
        let target = self.target.clone();
        self.runtime.block_on(async move {
            let (a, b) = futures::join!(source.engine_id(), target.info());
            matches!((a, b), (Ok(Some(a)), Ok(info)) if info.id.as_deref() == Some(a.as_str()))
        })
    }

    /// Copies the source volume `name` into a target volume `target_name`. The
    /// assistant keeps names; tests copy into a new name on one engine.
    pub fn copy_volume_as(&self, name: &str, target_name: &str) -> EngineStream<TransferEvent> {
        let (source, target) = (self.source.clone(), self.target.clone());
        let (name, target_name) = (name.to_string(), target_name.to_string());
        run(&self.runtime, &self.in_flight, move |events| async move {
            copy_volume(
                &source,
                &target,
                (&name, &target_name),
                Existing::Skip,
                &events,
            )
            .await
        })
    }

    /// Recreates the source container `id` as `target_name`, optionally from a
    /// snapshot. The assistant keeps names; tests copy into a new name.
    pub fn copy_container_as(
        &self,
        id: &str,
        target_name: &str,
        snapshot: bool,
    ) -> EngineStream<TransferEvent> {
        let (source, target) = (self.source.clone(), self.target.clone());
        let (id, target_name) = (id.to_string(), target_name.to_string());
        run(&self.runtime, &self.in_flight, move |events| async move {
            copy_container(&source, &target, &id, Some(&target_name), snapshot, &events).await
        })
    }

    /// Switches the source container `name` over into `target_name`, and copies each
    /// pair of source and target volumes again. The assistant keeps names; tests
    /// switch into a new name on one engine.
    pub fn switch_container_as(
        &self,
        name: &str,
        target_name: &str,
        volumes: &[(String, String)],
    ) -> EngineStream<TransferEvent> {
        let (source, target) = (self.source.clone(), self.target.clone());
        let (name, target_name) = (name.to_string(), target_name.to_string());
        let (volumes, local) = (volumes.to_vec(), self.target_is_local());
        let switching = self.switching.enter();
        run(&self.runtime, &self.in_flight, move |events| async move {
            let _switching = switching;
            let job = Job::container(&name, &target_name, volumes, local);
            switch_over(&source, &target, job, &events).await
        })
    }

    /// Rolls back [`Self::switch_container_as`].
    pub fn roll_back_container_as(&self, name: &str, target_name: &str) -> EngineFuture<()> {
        let (source, target) = (self.source.clone(), self.target.clone());
        let (name, target_name) = (name.to_string(), target_name.to_string());
        let local = self.target_is_local();
        runtime::spawn(self.runtime.handle(), async move {
            let job = Job::container(&name, &target_name, Vec::new(), local);
            roll_back(&source, &target, &job).await
        })
    }

    /// True if the target runs on this computer, so its published ports are on
    /// 127.0.0.1.
    fn target_is_local(&self) -> bool {
        matches!(
            self.target_endpoint,
            Endpoint::Unix(_) | Endpoint::NamedPipe(_)
        )
    }

    fn compose_cli(&self) -> impl Future<Output = Result<ComposeCli, String>> + Send + 'static {
        let endpoint = self.target_endpoint.clone();
        async move {
            let detect = tokio::task::spawn_blocking(move || ComposeCli::detect(&endpoint)).await;
            detect.unwrap_or_else(|error| Err(error.to_string()))
        }
    }
}

impl MigrationSession for DockerSession {
    fn source(&self) -> &str {
        &self.source_host
    }

    fn scan(&self) -> EngineFuture<MigrationPlan> {
        let source = self.source.clone();
        let label = self.source_host.clone();
        runtime::spawn(self.runtime.handle(), async move {
            scan::scan(&source, &label).await
        })
    }

    fn target_free_space(&self) -> EngineFuture<Option<u64>> {
        let target = self.target.clone();
        runtime::spawn(self.runtime.handle(), async move {
            Ok(disk::free_space(&target)
                .await
                .inspect_err(|error| tracing::warn!(%error, "cannot read the target's free space"))
                .ok())
        })
    }

    fn copy(&self, item: &MigrationItem, snapshot: bool) -> EngineStream<TransferEvent> {
        let (source, target) = (self.source.clone(), self.target.clone());
        let item = item.clone();
        let cli = matches!(item, MigrationItem::ComposeProject { .. }).then(|| self.compose_cli());
        run(&self.runtime, &self.in_flight, move |events| async move {
            match &item {
                MigrationItem::Network { name, .. } => {
                    copy_network(&source, &target, name, &events).await
                }
                MigrationItem::Volume { name, .. } => {
                    copy_volume(&source, &target, (name, name), Existing::Skip, &events).await
                }
                MigrationItem::Image { id, tags, size, .. } => {
                    copy_image(&source, &target, id, tags, *size, &events).await
                }
                MigrationItem::Container { id, .. } => {
                    copy_container(&source, &target, id, None, snapshot, &events).await
                }
                MigrationItem::ComposeProject {
                    name,
                    working_dir,
                    config_files,
                    files_exist,
                    containers,
                    ..
                } => {
                    let cli = match cli {
                        Some(cli) => cli.await,
                        None => Err("no Compose CLI".into()),
                    };
                    let project = Project {
                        name,
                        working_dir: working_dir.as_deref(),
                        config_files,
                        files_exist: *files_exist,
                        containers,
                    };
                    copy_project(&source, &target, cli, project, &events).await
                }
            }
        })
    }

    fn switch_over(&self, item: &MigrationItem, snapshot: bool) -> EngineStream<TransferEvent> {
        let (source, target) = (self.source.clone(), self.target.clone());
        let item = item.clone();
        let cli = matches!(item, MigrationItem::ComposeProject { .. }).then(|| self.compose_cli());
        let local = self.target_is_local();
        let switching = self.switching.enter();
        run(&self.runtime, &self.in_flight, move |events| async move {
            let _switching = switching;
            let cli = match cli {
                Some(cli) => cli.await,
                None => Err("no Compose CLI".into()),
            };
            let job =
                Job::for_item(&item, snapshot, cli, local).ok_or_else(|| not_running(&item))?;
            switch_over(&source, &target, job, &events).await
        })
    }

    fn roll_back(&self, item: &MigrationItem) -> EngineFuture<()> {
        let (source, target) = (self.source.clone(), self.target.clone());
        let item = item.clone();
        let local = self.target_is_local();
        runtime::spawn(self.runtime.handle(), async move {
            let cli = Err("a roll back needs no Compose CLI".into());
            let job = Job::for_item(&item, false, cli, local).ok_or_else(|| not_running(&item))?;
            roll_back(&source, &target, &job).await
        })
    }

    /// Waits for every switch-over to end, and for stopped copies up to
    /// [`CLEANUP_WAIT`], then removes the helpers. If a copy still runs then, the
    /// helpers stay: the next session removes them as leftovers.
    fn finish(&self) -> EngineFuture<()> {
        let (source, target) = (self.source.clone(), self.target.clone());
        let (in_flight, switching) = (self.in_flight.clone(), self.switching.clone());
        runtime::spawn(self.runtime.handle(), async move {
            switching.wait_idle(None).await;
            if !in_flight.wait_idle(Some(CLEANUP_WAIT)).await {
                tracing::warn!(
                    copies = in_flight.count(),
                    "copies still run at the end of a session; their helpers stay"
                );
                return Ok(());
            }
            source.clean_up().await;
            for id in helper::leftovers(&target).await.unwrap_or_default() {
                helper::remove(&target, &id).await;
            }
            Ok(())
        })
    }
}

fn not_running(item: &MigrationItem) -> EngineError {
    EngineError::Api(format!(
        "{} did not run in the old engine, so it cannot switch over.",
        item.label()
    ))
}

/// Runs one copy on `runtime` and reports its events. A skip becomes a
/// [`TransferEvent::Skipped`], and a failure the stream's error.
fn run<F, Fut>(runtime: &Runtime, in_flight: &InFlight, copy: F) -> EngineStream<TransferEvent>
where
    F: FnOnce(Events) -> Fut + Send + 'static,
    Fut: Future<Output = Result<Outcome, EngineError>> + Send + 'static,
{
    let guard = in_flight.enter();
    runtime::forward_to_end(runtime.handle(), move |events: Events| async move {
        let _guard = guard;
        match copy(events.clone()).await {
            Ok(Outcome::Copied) => {}
            Ok(Outcome::Skipped(reason)) => {
                events
                    .unbounded_send(Ok(TransferEvent::Skipped(reason)))
                    .ok();
            }
            Err(error) => {
                events.unbounded_send(Err(error)).ok();
            }
        }
    })
}

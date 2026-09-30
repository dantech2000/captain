//! Where the MCP server gets its data: the engine the settings choose, connected
//! on first use, and the crash tracker that the engine's events feed.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Instant;

use captain_core::store::{Crash, CrashTracker};
use captain_core::{Engine, EngineError, EngineHost, HostResources, HostStatus};
use futures::StreamExt;
use tokio::task::JoinHandle;

/// Connects to the engine. It blocks, so it runs off the async threads.
pub type Connect = Arc<dyn Fn() -> Result<Arc<dyn Engine>, String> + Send + Sync>;

pub struct Source {
    /// `Captain Engine` or `Other engine`.
    pub label: &'static str,
    /// Captain Engine's machine; `None` for another engine.
    host: Option<Arc<dyn EngineHost>>,
    connect: Connect,
    /// True when the settings turn on Kubernetes in Captain Engine.
    pub kubernetes: bool,
    engine: Mutex<Option<Arc<dyn Engine>>>,
    crashes: Arc<Mutex<CrashTracker>>,
    follower: Mutex<Option<JoinHandle<()>>>,
}

impl Source {
    pub fn new(
        label: &'static str,
        host: Option<Arc<dyn EngineHost>>,
        connect: Connect,
        kubernetes: bool,
    ) -> Self {
        Self {
            label,
            host,
            connect,
            kubernetes,
            engine: Mutex::new(None),
            crashes: Arc::default(),
            follower: Mutex::new(None),
        }
    }

    /// Captain Engine's state and resources, or `None` for another engine.
    pub async fn host_status(&self) -> Option<(HostStatus, HostResources)> {
        let host = self.host.clone()?;
        let resources = host.resources();
        // The Lima host runs `limactl`; keep it off the async threads.
        let status =
            tokio::task::spawn_blocking(move || futures::executor::block_on(host.status()))
                .await
                .ok()?
                .unwrap_or_else(|error| HostStatus::Failed(error.0));
        Some((status, resources))
    }

    /// Captain Engine's snapshots, when it has them.
    pub fn host(&self) -> Option<&Arc<dyn EngineHost>> {
        self.host.as_ref()
    }

    /// The engine, connecting on first use. The first connection also starts
    /// following its events for the crash tracker.
    pub async fn engine(&self) -> Result<Arc<dyn Engine>, String> {
        if let Some(engine) = lock(&self.engine).clone() {
            return Ok(engine);
        }
        let connect = self.connect.clone();
        let engine = tokio::task::spawn_blocking(move || connect())
            .await
            .map_err(|error| error.to_string())?
            .map_err(|why| self.unreachable(why))?;
        *lock(&self.engine) = Some(engine.clone());
        self.follow(engine.clone());
        Ok(engine)
    }

    /// Forgets the engine after `error`, so the next call connects again, and says
    /// what went wrong.
    pub fn failed(&self, error: EngineError) -> String {
        if matches!(error, EngineError::Unreachable(_)) {
            *lock(&self.engine) = None;
            if let Some(follower) = lock(&self.follower).take() {
                follower.abort();
            }
        }
        error.to_string()
    }

    /// The crash of `id` in the last minute, from the engine's events.
    pub fn crash(&self, id: &str) -> Option<Crash> {
        lock(&self.crashes).recent(id, Instant::now())
    }

    fn unreachable(&self, why: String) -> String {
        match self.host {
            Some(_) => format!(
                "Captain Engine does not answer ({why}). Start it in Captain, or run `captain start`."
            ),
            None => format!("The engine does not answer: {why}"),
        }
    }

    fn follow(&self, engine: Arc<dyn Engine>) {
        let crashes = self.crashes.clone();
        let follower = tokio::spawn(async move {
            let mut events = engine.events();
            while let Some(Ok(event)) = events.next().await {
                lock(&crashes).record(&event);
            }
        });
        if let Some(old) = lock(&self.follower).replace(follower) {
            old.abort();
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

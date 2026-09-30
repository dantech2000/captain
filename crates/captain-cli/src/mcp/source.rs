//! Where the MCP server gets its data: the engine the settings choose, connected
//! on first use, the crash tracker that the engine's events feed, the
//! `agent_tools` settings, and the activity log.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Instant;

use captain_core::agent_tools::{Activity, AgentToolsSettings, append_activity};
use captain_core::store::{Crash, CrashTracker};
use captain_core::{Engine, EngineError, EngineHost, HostResources, HostStatus, ProjectRunner};
use futures::StreamExt;
use serde_json::{Map, Value};
use tokio::task::JoinHandle;

/// The engine, and the `docker compose` CLI for it when there is one.
#[derive(Clone)]
pub struct Connected {
    pub engine: Arc<dyn Engine>,
    pub runner: Option<Arc<dyn ProjectRunner>>,
}

/// Connects to the engine. It blocks, so it runs off the async threads.
pub type Connect = Arc<dyn Fn() -> Result<Connected, String> + Send + Sync>;

/// Reads the `agent_tools` settings. The server calls it for every request, so a
/// change in Captain applies at once.
pub type ReadSettings = Arc<dyn Fn() -> AgentToolsSettings + Send + Sync>;

pub struct Source {
    /// `Captain Engine` or `Other engine`.
    pub label: &'static str,
    /// Captain Engine's machine; `None` for another engine.
    host: Option<Arc<dyn EngineHost>>,
    connect: Connect,
    /// True when the settings turn on Kubernetes in Captain Engine.
    pub kubernetes: bool,
    engine: Mutex<Option<Connected>>,
    settings: ReadSettings,
    /// `~/.captain/agent-activity.jsonl`, or `None` to keep no log.
    activity: Option<PathBuf>,
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
            settings: Arc::new(AgentToolsSettings::default),
            activity: None,
            crashes: Arc::default(),
            follower: Mutex::new(None),
        }
    }

    /// Reads the `agent_tools` settings with `read`. Without it, they are the
    /// defaults: every tool but `help` is off.
    pub fn with_settings(mut self, read: ReadSettings) -> Self {
        self.settings = read;
        self
    }

    /// Writes an entry for each tool call to `path`.
    pub fn with_activity(mut self, path: PathBuf) -> Self {
        self.activity = Some(path);
        self
    }

    /// The `agent_tools` settings now.
    pub fn agent_settings(&self) -> AgentToolsSettings {
        (self.settings)()
    }

    /// Adds a tool call to the activity log and to stderr.
    pub fn record(
        &self,
        client: &str,
        tool: &str,
        arguments: Map<String, Value>,
        ok: bool,
        result: &str,
    ) {
        tracing::info!(client, tool, ok, "tool call");
        let Some(path) = &self.activity else {
            return;
        };
        let entry = Activity::new(super::server::now(), client, tool, arguments, ok, result);
        if let Err(error) = append_activity(path, &entry) {
            tracing::warn!(%error, "cannot write {}", path.display());
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
        self.connected().await.map(|connected| connected.engine)
    }

    /// The `docker compose` CLI, for project actions and tasks.
    pub async fn runner(&self) -> Result<Arc<dyn ProjectRunner>, String> {
        self.connected().await?.runner.ok_or_else(|| {
            "Captain cannot find docker compose, so it cannot run project actions or tasks."
                .to_string()
        })
    }

    async fn connected(&self) -> Result<Connected, String> {
        if let Some(connected) = lock(&self.engine).clone() {
            return Ok(connected);
        }
        let connect = self.connect.clone();
        let connected = tokio::task::spawn_blocking(move || connect())
            .await
            .map_err(|error| error.to_string())?
            .map_err(|why| self.unreachable(why))?;
        *lock(&self.engine) = Some(connected.clone());
        self.follow(connected.engine.clone());
        Ok(connected)
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

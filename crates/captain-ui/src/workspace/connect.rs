use std::sync::Arc;
use std::time::Duration;

use captain_core::store::{ContainerStore, StatsBoard};
use captain_core::{Engine, EngineError, ProjectRunner};
use gpui_kit::*;

use super::{Connection, Workspace};

/// An engine, and the Compose runner when the `docker compose` CLI is installed.
pub type Connected = (Arc<dyn Engine>, Option<Arc<dyn ProjectRunner>>);

/// Connects to an engine. It may block, so the workspace runs it on a background thread.
pub type Connector = Box<dyn FnOnce() -> Result<Connected, EngineError> + Send>;

/// The workspace that started the last connection, and a count of connections. A
/// connection that finishes after a newer one started is thrown away.
struct LastConnect {
    workspace: WeakEntity<Workspace>,
    attempt: u64,
}

impl Global for LastConnect {}

/// The workspace that connected last, for views that have no workspace handle.
pub fn active_workspace(cx: &App) -> Option<Entity<Workspace>> {
    cx.try_global::<LastConnect>()?.workspace.upgrade()
}

fn is_current(attempt: u64, cx: &App) -> bool {
    cx.try_global::<LastConnect>()
        .is_some_and(|last| last.attempt == attempt)
}

impl Workspace {
    /// Connects in the background, then loads containers and follows events.
    pub fn connect(&mut self, connect: Connector, cx: &mut Context<Self>) {
        let attempt = cx
            .try_global::<LastConnect>()
            .map_or(0, |last| last.attempt)
            + 1;
        cx.set_global(LastConnect {
            workspace: cx.weak_entity(),
            attempt,
        });
        cx.spawn(async move |this, cx| {
            let connected = cx
                .background_executor()
                .spawn(async move { connect() })
                .await;
            let (engine, projects) = match connected {
                Ok(connected) => connected,
                Err(error) => {
                    this.update(cx, |this, cx| {
                        if is_current(attempt, cx) {
                            this.fail(error, cx);
                        }
                    })
                    .ok();
                    return;
                }
            };
            let info = engine.info().await;
            this.update(cx, |this, cx| {
                if !is_current(attempt, cx) {
                    return;
                }
                match info {
                    Ok(info) => {
                        this.connection = Connection::Connected(info);
                        this.engine = Some(engine);
                        this.set_project_runner(projects, cx);
                        this.reload(Duration::ZERO, cx);
                        this.watch_events(cx);
                        cx.notify();
                    }
                    Err(error) => this.fail(error, cx),
                }
            })
            .ok();
        })
        .detach();
    }

    /// Drops the engine, its tasks, and everything loaded from it, then connects again
    /// with `connect`. The pages see no engine, then the new one.
    pub fn reconnect(&mut self, connect: Connector, cx: &mut Context<Self>) {
        tracing::info!("reconnecting to the engine");
        self.engine = None;
        self.connection = Connection::Connecting;
        self.loaded = false;
        self.store = ContainerStore::default();
        self.stats = StatsBoard::default();
        self.selected = None;
        self.reload_task = None;
        self.events_task = None;
        self.stats_tasks.clear();
        self.pending.clear();
        self.projects = None;
        self.project_pending.clear();
        self.page_counts.clear();
        cx.notify();
        self.connect(connect, cx);
    }
}

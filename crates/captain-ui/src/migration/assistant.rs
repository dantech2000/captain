use std::sync::Arc;

use captain_core::migration::{
    MigrationBackend, MigrationPlan, MigrationRun, MigrationSession, SourceOption,
};
use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::backend::backend;
use crate::workspace::{Connection, Workspace};

/// The page the assistant shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Pick the engine to copy from.
    Choose,
    /// Connecting to it and reading what it holds.
    Loading,
    /// Pick what to copy.
    Review,
    /// Copying, item by item.
    Run,
    /// What was copied and what failed.
    Summary,
}

/// The Migration Assistant: copies data from another engine into the connected one.
/// See docs/adr/0009-migration.md.
pub struct MigrationAssistant {
    pub(super) backend: Option<Arc<dyn MigrationBackend>>,
    /// The connected engine, which receives the copies.
    pub(super) target: Option<String>,
    /// True while the workspace connects, as it does right after the first setup.
    pub(super) connecting: bool,
    pub(super) sources: Vec<SourceOption>,
    pub(super) custom: Entity<InputState>,
    pub(super) session: Option<Arc<dyn MigrationSession>>,
    pub(super) plan: Option<MigrationPlan>,
    /// Free bytes on the target's disk, if known.
    pub(super) free: Option<u64>,
    pub(super) run: MigrationRun,
    pub(super) stage: Stage,
    pub(super) error: Option<String>,
    /// The connect, scan, or copy that runs now. Dropping it cancels it.
    pub(super) task: Option<Task<()>>,
    _workspace: Subscription,
}

impl MigrationAssistant {
    pub fn new(workspace: &Entity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let backend = backend(cx);
        // The first setup opens the assistant while the workspace still connects to
        // the new engine, so the target arrives later.
        let follow = cx.observe(workspace, |this, workspace, cx| {
            if this.target.is_none() {
                this.follow(workspace.read(cx).connection());
                cx.notify();
            }
        });
        let custom =
            cx.new(|cx| InputState::new(window, cx).placeholder("unix:///path/to/docker.sock"));
        // Helpers left in either engine go when the assistant closes.
        cx.on_release(|this, cx| {
            if let Some(session) = this.session.take() {
                finish(session, cx);
            }
        })
        .detach();
        let mut assistant = Self {
            backend,
            target: None,
            connecting: false,
            sources: Vec::new(),
            custom,
            session: None,
            plan: None,
            free: None,
            run: MigrationRun::default(),
            stage: Stage::Choose,
            error: None,
            task: None,
            _workspace: follow,
        };
        assistant.follow(workspace.read(cx).connection());
        assistant
    }

    /// Takes the target, and the engines to copy from, from the workspace's
    /// connection.
    fn follow(&mut self, connection: &Connection) {
        self.connecting = matches!(connection, Connection::Connecting);
        if let Connection::Connected(info) = connection {
            self.sources = match &self.backend {
                Some(backend) => backend.sources(&info.endpoint),
                None => Vec::new(),
            };
            self.target = Some(info.endpoint.clone());
        }
    }

    /// The source endpoint of the open session.
    pub(super) fn source(&self) -> Option<&str> {
        self.session.as_ref().map(|session| session.source())
    }

    pub(super) fn fail(&mut self, error: String, stage: Stage, cx: &mut Context<Self>) {
        tracing::warn!(%error, "migration assistant");
        self.error = Some(error);
        self.stage = stage;
        self.task = None;
        cx.notify();
    }
}

/// Ends `session` in the background: waits for a stopped copy to clean up, removes
/// helpers, then drops the session and its runtime.
pub(super) fn finish(session: Arc<dyn MigrationSession>, cx: &App) {
    cx.background_executor()
        .spawn(async move {
            session.finish().await.ok();
            drop(session);
        })
        .detach();
}

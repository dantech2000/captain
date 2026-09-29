use std::sync::Arc;

use captain_core::settings::EngineChoice;
use captain_core::{EngineHost, HostResources, HostStatus};
use gpui_kit::*;

use super::{HostEvent, ProgressLog};
use crate::settings::{self, DetectedEndpoint};
use crate::workspace::Workspace;

/// Captain Engine as the views see it: the host, its status, and the progress of
/// the current start. Actions live in `host_actions.rs`.
pub struct HostModel {
    pub(super) host: Arc<dyn EngineHost>,
    /// True if Captain Engine can run on this computer, so it is the default choice.
    pub(super) available: bool,
    /// This computer's CPUs and memory, the upper limits for the resources.
    pub(super) machine: HostResources,
    pub(super) status: HostStatus,
    /// True until the first status arrives.
    pub(super) checking: bool,
    pub(super) log: ProgressLog,
    /// The workspace to reconnect when the engine starts.
    pub(super) workspace: WeakEntity<Workspace>,
    /// Open the Migration Assistant when the first setup finishes.
    pub(super) migrate_after_setup: bool,
    /// Other engines on this computer, for the setup screen.
    pub(super) detected: Vec<DetectedEndpoint>,
    /// Why the last start failed, until the next start.
    pub(super) last_error: Option<String>,
    pub(super) start_task: Option<Task<()>>,
    /// True while a stop or a reset runs.
    pub(super) stopping: bool,
    /// True when a stop interrupted the start, so its error is expected.
    pub(super) cancelled: bool,
    /// True while a snapshot step runs. Start, stop, reset, and resizing wait.
    pub(super) snapshotting: bool,
    pub(super) _poll: Option<Task<()>>,
}

impl EventEmitter<HostEvent> for HostModel {}

struct HostHandle(Entity<HostModel>);

impl Global for HostHandle {}

/// Installs Captain Engine. `available` says whether it can run here; `machine` is
/// this computer's CPUs and memory; `workspace` is reconnected each time the
/// engine starts.
pub fn init(
    cx: &mut App,
    host: Arc<dyn EngineHost>,
    available: bool,
    machine: HostResources,
    workspace: &Entity<Workspace>,
) -> Entity<HostModel> {
    let workspace = workspace.downgrade();
    let model = cx.new(|cx| {
        let mut model = HostModel {
            host,
            available,
            machine,
            status: HostStatus::Stopped,
            checking: true,
            log: ProgressLog::default(),
            workspace,
            migrate_after_setup: true,
            detected: Vec::new(),
            last_error: None,
            start_task: None,
            stopping: false,
            cancelled: false,
            snapshotting: false,
            _poll: None,
        };
        model.rescan(cx);
        model.poll(cx);
        model
    });
    cx.set_global(HostHandle(model.clone()));
    model
}

/// The host model, once the app installed it.
pub fn host_model(cx: &App) -> Option<Entity<HostModel>> {
    cx.try_global::<HostHandle>().map(|handle| handle.0.clone())
}

/// True if the settings choose Captain Engine.
pub fn uses_captain(cx: &App) -> bool {
    host_model(cx).is_some_and(|model| model.read(cx).uses_captain(cx))
}

/// The Captain Engine endpoint when the settings choose it, else `None`.
pub fn captain_endpoint(cx: &App) -> Option<String> {
    let model = host_model(cx)?;
    let model = model.read(cx);
    model
        .uses_captain(cx)
        .then(|| model.host.endpoint())
        .flatten()
}

/// Captain Engine's endpoint, whether or not the settings choose it.
pub fn captain_socket(cx: &App) -> Option<String> {
    host_model(cx)?.read(cx).host.endpoint()
}

impl HostModel {
    pub fn status(&self) -> &HostStatus {
        &self.status
    }

    /// True until the first status check finishes.
    pub fn is_checking(&self) -> bool {
        self.checking
    }

    /// False for a host Captain does not control, and while a snapshot step runs.
    pub fn can_control(&self) -> bool {
        self.host.can_control() && !self.snapshotting
    }

    /// Why the last start failed, until the next start.
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn log(&self) -> &ProgressLog {
        &self.log
    }

    pub fn resources(&self) -> HostResources {
        self.host.resources()
    }

    /// This computer's CPUs and memory.
    pub fn machine(&self) -> HostResources {
        self.machine
    }

    pub fn detected(&self) -> &[DetectedEndpoint] {
        &self.detected
    }

    pub fn migrate_after_setup(&self) -> bool {
        self.migrate_after_setup
    }

    pub fn set_migrate_after_setup(&mut self, migrate: bool, cx: &mut Context<Self>) {
        self.migrate_after_setup = migrate;
        cx.notify();
    }

    /// The engine choice from the settings, with this computer's default.
    pub fn choice(&self, cx: &App) -> EngineChoice {
        settings::current(cx).engine_choice(self.available)
    }

    pub fn uses_captain(&self, cx: &App) -> bool {
        self.choice(cx) == EngineChoice::Captain
    }

    /// Looks for other engines again.
    pub fn rescan(&mut self, cx: &mut Context<Self>) {
        self.detected = settings::engine_source(cx)
            .map(|source| source.detected())
            .unwrap_or_default();
        cx.notify();
    }
}

use std::path::PathBuf;
use std::sync::Arc;

use captain_core::HostStatus;
use captain_core::diagnostics::{Check, Facts, MachineFacts, Platform, evaluate, failure_count};
use gpui_kit::*;

use super::engine_probe;
use crate::engine_host::host_model;
use crate::workspace::{Connection, Workspace};

/// What the app gives the Diagnostics page.
pub struct DiagnosticsSetup {
    /// Gathers the facts about this computer. It blocks, so it runs in the background.
    pub probe: Arc<dyn Fn() -> MachineFacts + Send + Sync>,
    /// The folder with Captain's log files.
    pub log_dir: Option<PathBuf>,
    /// Captain Engine's instance folder.
    pub engine_dir: Option<PathBuf>,
    /// Raises or lowers the log level at once.
    pub set_debug_logging: Arc<dyn Fn(bool) + Send + Sync>,
}

/// What a change looks like to the checks: the connection state and Captain
/// Engine's status. The checks run again when it changes.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Trigger {
    connection: u8,
    captain: Option<HostStatus>,
}

/// The latest check results. They run at launch, when the engine connection or
/// Captain Engine's status changes, and on "Run again".
pub struct DiagnosticsModel {
    pub(super) setup: DiagnosticsSetup,
    workspace: WeakEntity<Workspace>,
    checks: Vec<Check>,
    /// The last facts about this computer. A connection change reuses them.
    machine: Option<MachineFacts>,
    /// When the last run finished, as `HH:MM`.
    checked_at: Option<SharedString>,
    trigger: Option<Trigger>,
    task: Option<Task<()>>,
    /// Another run was asked for while one ran. `Some(true)` probes the machine too.
    queued: Option<bool>,
    _subscriptions: Vec<Subscription>,
}

struct DiagnosticsHandle(Entity<DiagnosticsModel>);

impl Global for DiagnosticsHandle {}

/// Installs the Diagnostics model and runs the checks for the first time.
pub fn init(cx: &mut App, workspace: &Entity<Workspace>, setup: DiagnosticsSetup) {
    let model = cx.new(|cx: &mut Context<DiagnosticsModel>| {
        let mut subscriptions = vec![
            cx.observe(workspace, |model: &mut DiagnosticsModel, _, cx| {
                model.follow(cx)
            }),
        ];
        subscriptions.extend(host_model(cx).map(|host| {
            cx.observe(&host, |model: &mut DiagnosticsModel, _, cx| {
                model.follow(cx)
            })
        }));
        let mut model = DiagnosticsModel {
            setup,
            workspace: workspace.downgrade(),
            checks: Vec::new(),
            machine: None,
            checked_at: None,
            trigger: None,
            task: None,
            queued: None,
            _subscriptions: subscriptions,
        };
        model.follow(cx);
        model
    });
    cx.set_global(DiagnosticsHandle(model));
}

pub fn diagnostics_model(cx: &App) -> Option<Entity<DiagnosticsModel>> {
    cx.try_global::<DiagnosticsHandle>()
        .map(|handle| handle.0.clone())
}

/// The number of failed checks, for the sidebar badge.
pub fn failures(cx: &App) -> usize {
    diagnostics_model(cx).map_or(0, |model| failure_count(&model.read(cx).checks))
}

/// Captain Engine's status when the settings choose it.
fn captain_status(cx: &App) -> Option<HostStatus> {
    let host = host_model(cx)?;
    let host = host.read(cx);
    if !host.uses_captain(cx) {
        return None;
    }
    Some(if host.is_checking() {
        HostStatus::Starting
    } else {
        host.status().clone()
    })
}

impl DiagnosticsModel {
    pub fn checks(&self) -> &[Check] {
        &self.checks
    }

    /// Captain Engine's instance folder, for the Show engine files fix.
    pub fn engine_dir(&self) -> Option<PathBuf> {
        self.setup.engine_dir.clone()
    }

    pub fn is_running(&self) -> bool {
        self.task.is_some()
    }

    pub fn checked_at(&self) -> Option<SharedString> {
        self.checked_at.clone()
    }

    /// Runs the checks when the connection or Captain Engine's status changed.
    fn follow(&mut self, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        let connection = match workspace.read(cx).connection() {
            Connection::Connecting => 0,
            Connection::Connected(_) => 1,
            Connection::Failed(_) => 2,
        };
        let trigger = Some(Trigger {
            connection,
            captain: captain_status(cx),
        });
        if trigger != self.trigger {
            let first = self.trigger.is_none();
            self.trigger = trigger;
            self.run(first, cx);
        }
    }

    /// Runs every check in the background. `machine` probes this computer again;
    /// otherwise only the engine is asked again.
    pub fn run(&mut self, machine: bool, cx: &mut Context<Self>) {
        if self.task.is_some() {
            self.queued = Some(self.queued.unwrap_or(false) || machine);
            return;
        }
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        let (connection, engine) = {
            let workspace = workspace.read(cx);
            (workspace.connection().clone(), workspace.engine())
        };
        let captain = captain_status(cx);
        let cached = self.machine.clone().filter(|_| !machine);
        let probe = self.setup.probe.clone();
        let executor = cx.background_executor().clone();
        self.task = Some(cx.spawn(async move |this, cx| {
            let machine = async {
                match cached {
                    Some(facts) => facts,
                    None => executor.spawn(async move { probe() }).await,
                }
            };
            let engine = engine_probe::probe(connection, engine, executor.clone());
            let (engine, machine) = futures::join!(engine, machine);
            let facts = Facts {
                platform: Platform::current(),
                captain_engine: captain,
                engine,
                machine,
            };
            this.update(cx, |model, cx| model.finish(facts, cx)).ok();
        }));
        cx.notify();
    }

    fn finish(&mut self, facts: Facts, cx: &mut Context<Self>) {
        self.task = None;
        self.checks = evaluate(&facts);
        self.machine = Some(facts.machine);
        self.checked_at = Some(chrono::Local::now().format("%H:%M").to_string().into());
        for check in &self.checks {
            tracing::debug!(check = ?check.id, state = ?check.state, detail = %check.detail);
        }
        if let Some(machine) = self.queued.take() {
            self.run(machine, cx);
        }
        cx.notify();
    }
}

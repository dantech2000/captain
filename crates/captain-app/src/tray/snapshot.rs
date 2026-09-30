use std::collections::HashMap;

use captain_core::HostStatus;
use captain_core::diagnostics::Check;
use captain_core::format::bytes_label;
use captain_core::kubernetes::KubeContexts;
use captain_core::model::Container;
use captain_core::problems::{ExitFacts, Problem, first_problem};
use captain_core::store::Crash;
use captain_ui::{Connection, HostSummary, Workspace};

use super::entries::{ContainerEntry, HostEntry, KubeEntry};

/// The engine state that the icon and the first status line show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineStatus {
    Starting,
    Running,
    Stopped,
    /// Captain Engine did not start, or a container is restarting or unhealthy.
    NeedsAttention,
}

impl EngineStatus {
    pub fn of(connection: &Connection) -> Self {
        match connection {
            Connection::Connecting => Self::Starting,
            Connection::Connected(_) => Self::Running,
            Connection::Failed(_) => Self::Stopped,
        }
    }

    /// With Captain Engine the host decides; a running host counts as running once
    /// the workspace has connected to it.
    pub fn of_host(host: &HostStatus, connection: &Connection) -> Self {
        match (host, connection) {
            (HostStatus::Running, Connection::Connected(_)) => Self::Running,
            (HostStatus::Running, Connection::Connecting) => Self::Starting,
            (HostStatus::Starting | HostStatus::Stopping, _) => Self::Starting,
            (HostStatus::Failed(_), _) => Self::NeedsAttention,
            _ => Self::Stopped,
        }
    }
}

/// The part of the workspace that the menu depends on. The tray rebuilds the menu
/// only when this changes, so stats samples do not rebuild it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraySnapshot {
    pub engine: EngineStatus,
    /// The first line: the engine, its state, and while it runs its CPUs and the
    /// memory in use. Only this line follows the stats; it changes in place.
    pub status_line: String,
    /// In the store's order: active containers first, then by name.
    pub containers: Vec<ContainerEntry>,
    /// Captain Engine, or `None` when Captain uses another engine.
    pub host: Option<HostEntry>,
    /// Kubernetes, while Captain Engine is the engine and Captain controls it.
    pub kubernetes: Option<KubeEntry>,
    /// The contexts in the user's kubeconfig, for the Kubernetes Contexts submenu.
    pub contexts: KubeContexts,
    /// The worst problem, with the fixes the menu offers for it.
    pub problem: Option<Problem>,
}

impl TraySnapshot {
    /// A snapshot without crashes, checks, or Captain Engine. Containers count only
    /// while the engine runs.
    #[cfg(test)]
    pub fn new(engine: EngineStatus, containers: &[Container]) -> Self {
        let scan = Scan {
            containers,
            crash: &|_| None,
            failure: None,
            checks: &[],
            facts: &HashMap::new(),
        };
        scan.snapshot(engine, format!("Captain Engine: {engine:?}"), None)
    }

    /// `checks` are the diagnostics checks; `facts` is what `inspect` said about each
    /// crashing container.
    pub fn of(
        workspace: &Workspace,
        host: Option<&HostSummary>,
        checks: &[Check],
        facts: &HashMap<String, ExitFacts>,
    ) -> Self {
        let connection = workspace.connection();
        let engine = host.map_or_else(
            || EngineStatus::of(connection),
            |host| EngineStatus::of_host(&host.status, connection),
        );
        let used = workspace.stats().total_memory();
        let line = status_line(connection, used, host.map(|host| &host.status));
        // Hidden Kubernetes containers do not count, as on the Containers page.
        let shown = shown(workspace);
        let failure = host.and_then(|host| match &host.status {
            HostStatus::Failed(why) => Some(why.as_str()),
            _ => None,
        });
        // A crash loop runs most of the time between restarts; the recent crashes
        // from the event stream keep the problem and the icon's dot steady.
        let scan = Scan {
            containers: &shown,
            crash: &|id| workspace.recent_crash(id),
            failure,
            checks,
            facts,
        };
        let host = host.map(|host| HostEntry {
            status: host.status.clone(),
            can_control: host.can_control,
        });
        scan.snapshot(engine, line, host)
    }

    /// True when the menus of both snapshots differ at most in the status line.
    pub fn same_menu(&self, other: &Self) -> bool {
        self.engine == other.engine
            && self.containers == other.containers
            && self.host == other.host
            && self.kubernetes == other.kubernetes
            && self.contexts == other.contexts
            && self.problem == other.problem
    }

    /// The state the menu bar icon shows: a running engine needs attention while a
    /// container is restarting or unhealthy.
    pub fn icon(&self) -> EngineStatus {
        let sick = self
            .problem
            .as_ref()
            .is_some_and(|problem| problem.container().is_some());
        match self.engine {
            EngineStatus::Running if sick => EngineStatus::NeedsAttention,
            engine => engine,
        }
    }

    /// The number of running, paused, or restarting containers.
    pub fn active_count(&self) -> usize {
        self.containers.iter().filter(|c| c.is_active()).count()
    }
}

/// What the snapshot reads the problem and the container list from.
struct Scan<'a> {
    containers: &'a [Container],
    crash: &'a dyn Fn(&str) -> Option<Crash>,
    /// Why Captain Engine did not start.
    failure: Option<&'a str>,
    checks: &'a [Check],
    facts: &'a HashMap<String, ExitFacts>,
}

impl Scan<'_> {
    fn snapshot(
        &self,
        engine: EngineStatus,
        status_line: String,
        host: Option<HostEntry>,
    ) -> TraySnapshot {
        let running = engine == EngineStatus::Running;
        // Containers count only while the engine runs; a failed engine keeps its old
        // list. A stopped engine is no problem, though some checks fail then.
        let containers: &[Container] = if running { self.containers } else { &[] };
        let checks = if running || self.failure.is_some() {
            self.checks
        } else {
            &[]
        };
        let problem = first_problem(self.failure, checks, containers, self.facts, self.crash);
        TraySnapshot {
            engine,
            status_line,
            containers: containers
                .iter()
                .map(|c| ContainerEntry::of(c, (self.crash)(&c.id).is_some()))
                .collect(),
            host,
            kubernetes: None,
            contexts: KubeContexts::default(),
            problem,
        }
    }
}

/// "Captain Engine: Running · 5 CPUs · 60 MB of 5.8 GB". `used` is the memory the
/// containers use; `host` is Captain Engine's status when the settings choose it.
fn status_line(connection: &Connection, used: u64, host: Option<&HostStatus>) -> String {
    let usage = match connection {
        Connection::Connected(info) => Some(format!(
            "Running \u{b7} {} CPUs \u{b7} {} of {}",
            info.cpus,
            bytes_label(used),
            bytes_label(info.memory_bytes)
        )),
        _ => None,
    };
    let (name, state) = match (host, connection) {
        (Some(status), _) => {
            let state = match status {
                HostStatus::Running => usage.unwrap_or_else(|| "Connecting".into()),
                HostStatus::NotInstalled(_) => "Needs Lima".into(),
                HostStatus::Failed(_) => "Did not start".into(),
                status => status.label().into(),
            };
            ("Captain Engine", state)
        }
        (None, Connection::Connected(info)) => (
            captain_ui::engine_name(&info.endpoint),
            usage.unwrap_or_default(),
        ),
        (None, Connection::Connecting) => ("Engine", "Connecting".into()),
        (None, Connection::Failed(_)) => ("Engine", "Not reachable".into()),
    };
    format!("{name}: {state}")
}

/// The containers the main window shows.
fn shown(workspace: &Workspace) -> Vec<Container> {
    let store = workspace.store();
    store.shown(workspace.show_kubernetes()).cloned().collect()
}

#[cfg(test)]
mod tests;

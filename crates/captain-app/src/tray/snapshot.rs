use std::collections::HashMap;

use captain_core::HostStatus;
use captain_core::kubernetes::KubeContexts;
use captain_core::model::{Container, ContainerState};
use captain_core::problems::{Problem, first_problem};
use captain_ui::{Connection, HostSummary, Workspace};

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

    pub fn label(self) -> &'static str {
        match self {
            Self::Starting => "Captain Engine is starting",
            Self::Running => "Captain Engine is running",
            Self::Stopped => "Captain Engine is stopped",
            Self::NeedsAttention => "Captain Engine needs attention",
        }
    }
}

/// One container, with only what the menu shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerEntry {
    pub id: String,
    pub name: String,
    pub state: ContainerState,
    pub project: Option<String>,
    pub ports: Vec<u16>,
}

impl ContainerEntry {
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }
}

/// Captain Engine's state, when the settings choose it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostEntry {
    pub status: HostStatus,
    pub can_control: bool,
}

impl HostEntry {
    /// The first line of the menu.
    pub fn label(&self) -> &'static str {
        match self.status {
            HostStatus::Running => "Captain Engine is running",
            HostStatus::Starting => "Captain Engine is starting",
            HostStatus::Stopping => "Captain Engine is stopping",
            HostStatus::Stopped => "Captain Engine is stopped",
            HostStatus::NotCreated => "Captain Engine is not set up",
            HostStatus::NotInstalled(_) => "Captain Engine needs Lima",
            HostStatus::Failed(_) => "Captain Engine did not start",
        }
    }
}

/// The part of the workspace that the menu depends on. The tray rebuilds the menu
/// only when this changes, so stats samples do not rebuild it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraySnapshot {
    pub engine: EngineStatus,
    /// In the store's order: active containers first, then by name.
    pub containers: Vec<ContainerEntry>,
    /// Captain Engine, or `None` when Captain uses another engine.
    pub host: Option<HostEntry>,
    /// The contexts in the user's kubeconfig, for the Kubernetes Contexts submenu.
    pub contexts: KubeContexts,
    /// The worst container problem, in the popover's words.
    pub problem: Option<String>,
}

impl TraySnapshot {
    /// Containers count only while the engine runs; a failed engine keeps its old list.
    pub fn new(engine: EngineStatus, containers: &[Container]) -> Self {
        // The popover also weighs diagnostics checks and why a container exited;
        // the menu has neither, so it names the container problem alone.
        let problem = (engine == EngineStatus::Running)
            .then(|| first_problem(None, &[], containers, &HashMap::new(), &|_| None))
            .flatten()
            .as_ref()
            .map(Problem::line);
        let containers = match engine {
            EngineStatus::Running => containers
                .iter()
                .map(|container| ContainerEntry {
                    id: container.id.clone(),
                    name: container.name.clone(),
                    state: container.state,
                    project: container.compose_project.clone(),
                    ports: container.published_ports(),
                })
                .collect(),
            _ => Vec::new(),
        };
        Self {
            engine,
            containers,
            host: None,
            contexts: KubeContexts::default(),
            problem,
        }
    }

    pub fn of(workspace: &Workspace, host: Option<&HostSummary>) -> Self {
        let connection = workspace.connection();
        let engine = host.map_or_else(
            || EngineStatus::of(connection),
            |host| EngineStatus::of_host(&host.status, connection),
        );
        // Hidden Kubernetes containers do not count, as on the Containers page.
        let shown = shown(workspace);
        let mut snapshot = Self {
            host: host.map(|host| HostEntry {
                status: host.status.clone(),
                can_control: host.can_control,
            }),
            ..Self::new(engine, &shown)
        };
        // A crash loop runs most of the time between restarts; the recent crashes
        // from the event stream keep the icon's dot steady through it.
        if engine == EngineStatus::Running {
            snapshot.problem = first_problem(None, &[], &shown, &HashMap::new(), &|id| {
                workspace.recent_crash(id)
            })
            .as_ref()
            .map(Problem::line);
        }
        snapshot
    }

    /// The state the menu bar icon shows: a running engine needs attention while a
    /// container is restarting or unhealthy.
    pub fn icon(&self) -> EngineStatus {
        match self.engine {
            EngineStatus::Running if self.problem.is_some() => EngineStatus::NeedsAttention,
            engine => engine,
        }
    }

    /// The number of running, paused, or restarting containers.
    pub fn active_count(&self) -> usize {
        self.containers.iter().filter(|c| c.is_active()).count()
    }
}

/// The containers the main window shows.
fn shown(workspace: &Workspace) -> Vec<Container> {
    let store = workspace.store();
    store.shown(workspace.show_kubernetes()).cloned().collect()
}

#[cfg(test)]
mod tests;

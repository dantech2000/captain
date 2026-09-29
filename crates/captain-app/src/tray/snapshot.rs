use captain_core::model::{Container, ContainerState};
use captain_ui::{Connection, Workspace};

/// The engine state that the icon and the first status line show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineStatus {
    Starting,
    Running,
    Stopped,
}

impl EngineStatus {
    pub fn of(connection: &Connection) -> Self {
        match connection {
            Connection::Connecting => Self::Starting,
            Connection::Connected(_) => Self::Running,
            Connection::Failed(_) => Self::Stopped,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Starting => "Captain Engine is starting",
            Self::Running => "Captain Engine is running",
            Self::Stopped => "Captain Engine is stopped",
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

/// The part of the workspace that the menu depends on. The tray rebuilds the menu
/// only when this changes, so stats samples do not rebuild it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraySnapshot {
    pub engine: EngineStatus,
    /// In the store's order: active containers first, then by name.
    pub containers: Vec<ContainerEntry>,
}

impl TraySnapshot {
    /// Containers count only while the engine runs; a failed engine keeps its old list.
    pub fn new(engine: EngineStatus, containers: &[Container]) -> Self {
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
        Self { engine, containers }
    }

    pub fn of(workspace: &Workspace) -> Self {
        Self::new(
            EngineStatus::of(workspace.connection()),
            workspace.store().containers(),
        )
    }

    /// The number of running, paused, or restarting containers.
    pub fn active_count(&self) -> usize {
        self.containers.iter().filter(|c| c.is_active()).count()
    }
}

#[cfg(test)]
mod tests;

use captain_core::EngineError;
use captain_core::model::{ContainerAction, EngineEvent, ProjectAction};
use gpui_kit::EventEmitter;
use gpui_kit::component::notification::Notification;

/// Something the window should tell the user about. The workspace has no window, so
/// the shell subscribes and turns these into notifications.
#[derive(Debug, Clone)]
pub enum WorkspaceEvent {
    /// The engine refused or failed a container action.
    ActionFailed {
        id: String,
        name: String,
        action: ContainerAction,
        error: EngineError,
    },
    /// A container was deleted.
    ContainerRemoved { name: String },
    /// A `docker compose` command on a project failed.
    ProjectFailed {
        project: String,
        action: ProjectAction,
        error: EngineError,
    },
    /// A `docker compose` command on a project finished.
    ProjectDone {
        project: String,
        action: ProjectAction,
    },
}

impl WorkspaceEvent {
    /// The toast for this event. A failure stays until the user closes it, and a
    /// repeat failure on the same container or project replaces the earlier toast.
    pub fn notification(&self) -> Notification {
        match self {
            Self::ActionFailed {
                id,
                name,
                action,
                error,
            } => Notification::error(error.to_string())
                .title(format!("{} {name} failed", action.label()))
                .id1::<WorkspaceEvent>(id.clone())
                .autohide(false),
            Self::ContainerRemoved { name } => Notification::success(format!("Deleted {name}.")),
            Self::ProjectFailed {
                project,
                action,
                error,
            } => Notification::error(error.to_string())
                .title(format!("{} {project} failed", action.label()))
                .id1::<WorkspaceEvent>(format!("project-{project}"))
                .autohide(false),
            Self::ProjectDone { project, action } => {
                Notification::success(action.done_message(project))
            }
        }
    }
}

/// Every engine event the workspace receives. Pages subscribe to this instead of
/// opening their own event streams.
impl EventEmitter<EngineEvent> for super::Workspace {}

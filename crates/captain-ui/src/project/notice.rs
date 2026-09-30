use gpui_kit::component::notification::Notification;

use crate::widgets::error_notification;

/// Something the Project page tells the user in a toast. The shell subscribes and
/// shows it, as it does for workspace events.
#[derive(Debug, Clone)]
pub enum ProjectNotice {
    Copied {
        address: String,
    },
    MemoryRaised {
        name: String,
        limit: String,
    },
    /// Staged changes applied to one container, for example `memory limit 512 MB`.
    Updated {
        name: String,
        summary: String,
    },
    Failed {
        title: String,
        error: String,
    },
    /// Save and apply or Rebuild failed. The editor shows why, so the toast hides
    /// itself and never covers the editor's buttons for long.
    EditorFailed {
        title: String,
    },
    TaskDone {
        task: String,
        exit_code: i32,
    },
    /// `up` ran after Save and apply, or `up --build` for one service.
    Applied {
        project: String,
        rebuilt: Option<String>,
    },
}

impl ProjectNotice {
    pub fn notification(&self) -> Notification {
        match self {
            Self::Copied { address } => Notification::success(format!("Copied {address}.")),
            Self::MemoryRaised { name, limit } => {
                Notification::success(format!("Set the memory limit of {name} to {limit}."))
            }
            Self::Updated { name, summary } => {
                Notification::success(format!("Updated {name}: {summary}."))
            }
            Self::Failed { title, error } => error_notification(title.clone(), error.clone()),
            Self::EditorFailed { title } => {
                error_notification(title.clone(), "The editor and the project log show why.")
            }
            Self::Applied {
                project,
                rebuilt: None,
            } => Notification::success(format!(
                "Applied the files of {project}. The project log shows Compose's output."
            )),
            Self::Applied {
                rebuilt: Some(service),
                ..
            } => Notification::success(format!("Rebuilt {service} and recreated its container.")),
            Self::TaskDone { task, exit_code: 0 } => {
                Notification::success(format!("The task {task} finished."))
            }
            Self::TaskDone { task, exit_code } => error_notification(
                format!("{task} failed"),
                format!("The task {task} exited with {exit_code}."),
            ),
        }
    }
}

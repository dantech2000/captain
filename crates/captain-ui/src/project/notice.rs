use gpui_kit::component::notification::Notification;

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
            Self::Failed { title, error } => Notification::error(error.clone())
                .title(title.clone())
                .autohide(false),
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
            Self::TaskDone { task, exit_code } => {
                Notification::error(format!("The task {task} exited with {exit_code}."))
                    .title(format!("{task} failed"))
            }
        }
    }
}

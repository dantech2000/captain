use gpui_kit::component::notification::Notification;

/// Something the Project page tells the user in a toast. The shell subscribes and
/// shows it, as it does for workspace events.
#[derive(Debug, Clone)]
pub enum ProjectNotice {
    Copied { address: String },
    MemoryRaised { name: String, limit: String },
    Failed { title: String, error: String },
    TaskDone { task: String, exit_code: i32 },
}

impl ProjectNotice {
    pub fn notification(&self) -> Notification {
        match self {
            Self::Copied { address } => Notification::success(format!("Copied {address}.")),
            Self::MemoryRaised { name, limit } => {
                Notification::success(format!("Set the memory limit of {name} to {limit}."))
            }
            Self::Failed { title, error } => Notification::error(error.clone())
                .title(title.clone())
                .autohide(false),
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

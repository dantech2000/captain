use gpui_kit::component::notification::Notification;

/// Something about Captain Engine that the window should act on. The host model
/// has no window, so the shell subscribes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostEvent {
    /// A start, stop, or reset failed.
    Failed {
        action: &'static str,
        message: String,
    },
    /// The first setup finished and the user asked to bring data along.
    OpenMigration,
}

impl HostEvent {
    /// The toast for this event, if it has one.
    pub fn notification(&self) -> Option<Notification> {
        match self {
            Self::Failed { action, message } => Some(
                Notification::error(message.clone())
                    .title(format!("Captain Engine: {action} failed"))
                    .id1::<HostEvent>(*action)
                    .autohide(false),
            ),
            Self::OpenMigration => None,
        }
    }
}

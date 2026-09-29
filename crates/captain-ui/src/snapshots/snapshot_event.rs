use gpui_kit::component::notification::Notification;

/// The end of a snapshot step, for a toast. The model has no window, so the view
/// subscribes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotEvent {
    Done(String),
    Failed {
        action: &'static str,
        message: String,
    },
}

impl SnapshotEvent {
    pub fn notification(&self) -> Notification {
        match self {
            Self::Done(message) => Notification::success(message.clone()),
            Self::Failed { action, message } => Notification::error(message.clone())
                .title(format!("Snapshot: {action} failed"))
                .id1::<SnapshotEvent>(*action)
                .autohide(false),
        }
    }
}

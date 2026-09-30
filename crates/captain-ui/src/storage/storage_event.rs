use gpui_kit::component::notification::Notification;

use crate::widgets::error_notification;

/// The end of a cleanup, for a toast. The model has no window, so the view
/// subscribes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageEvent {
    Done(String),
    Failed(String),
}

impl StorageEvent {
    pub fn notification(&self) -> Notification {
        match self {
            Self::Done(message) => Notification::success(message.clone()),
            Self::Failed(message) => {
                error_notification("Cleanup failed", message.clone()).id::<StorageEvent>()
            }
        }
    }
}

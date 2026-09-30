use gpui_kit::component::notification::Notification;

use crate::widgets::error_notification;

/// A Kubernetes action that failed. The model has no window, so the shell
/// subscribes and shows a toast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubeEvent {
    pub action: &'static str,
    pub message: String,
}

impl KubeEvent {
    pub fn notification(&self) -> Notification {
        error_notification(
            format!("Kubernetes: {} failed", self.action),
            self.message.clone(),
        )
        .id1::<KubeEvent>(self.action)
    }
}

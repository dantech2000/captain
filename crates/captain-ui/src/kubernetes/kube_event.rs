use gpui_kit::component::notification::Notification;

/// A Kubernetes action that failed. The model has no window, so the shell
/// subscribes and shows a toast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubeEvent {
    pub action: &'static str,
    pub message: String,
}

impl KubeEvent {
    pub fn notification(&self) -> Notification {
        Notification::error(self.message.clone())
            .title(format!("Kubernetes: {} failed", self.action))
            .id1::<KubeEvent>(self.action)
            .autohide(false)
    }
}

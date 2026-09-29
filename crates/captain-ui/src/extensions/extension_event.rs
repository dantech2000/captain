use captain_core::extension::ExtensionCandidate;
use gpui_kit::component::notification::Notification;

/// What the model tells the page. It has no window, so the page subscribes: for
/// toasts, and to ask before an install.
#[derive(Debug, Clone)]
pub enum ExtensionEvent {
    /// The image is an extension; ask the user before installing it.
    Confirm(Box<ExtensionCandidate>),
    /// An install finished; the page clears the image field.
    Installed(String),
    Done(String),
    Failed {
        action: &'static str,
        message: String,
    },
}

impl ExtensionEvent {
    pub fn notification(&self) -> Option<Notification> {
        match self {
            Self::Confirm(_) => None,
            Self::Installed(message) | Self::Done(message) => {
                Some(Notification::success(message.clone()))
            }
            Self::Failed { action, message } => Some(
                Notification::error(message.clone())
                    .title(format!("Extension: {action} failed"))
                    .id1::<ExtensionEvent>(*action)
                    .autohide(false),
            ),
        }
    }
}

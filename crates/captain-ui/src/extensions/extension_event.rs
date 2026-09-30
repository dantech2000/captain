use captain_core::extension::{ExtensionCandidate, ExtensionUpdate};
use gpui_kit::component::notification::Notification;

use crate::widgets::error_notification;

/// What the model tells the page. It has no window, so the page subscribes: for
/// toasts, and to ask before an install.
#[derive(Debug, Clone)]
pub enum ExtensionEvent {
    /// The image is an extension; ask the user before installing it.
    Confirm(Box<ExtensionCandidate>),
    /// A newer image exists; ask the user before updating.
    ConfirmUpdate(Box<ExtensionUpdate>),
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
            Self::Confirm(_) | Self::ConfirmUpdate(_) => None,
            Self::Installed(message) | Self::Done(message) => {
                Some(Notification::success(message.clone()))
            }
            Self::Failed { action, message } => Some(
                error_notification(format!("Extension: {action} failed"), message.clone())
                    .id1::<ExtensionEvent>(*action),
            ),
        }
    }
}

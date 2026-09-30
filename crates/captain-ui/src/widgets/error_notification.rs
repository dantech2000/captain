//! The error toast every page uses. Like the other toasts it hides after
//! gpui-kit's five seconds, and the timer pauses while the pointer is over the
//! toasts. A long message shows its first lines; captain.log keeps the whole text.

use gpui_kit::component::notification::{Notification, NotificationType};
use gpui_kit::*;

/// The lines of the message a toast shows.
const MESSAGE_LINES: usize = 3;

/// An error toast titled `title`, with `message` cut to a few lines. The full
/// message goes to the log.
pub fn error_notification(
    title: impl Into<SharedString>,
    message: impl Into<SharedString>,
) -> Notification {
    let (title, message) = (title.into(), message.into());
    tracing::warn!(%title, %message, "error toast");
    Notification::new()
        .with_type(NotificationType::Error)
        .title(title)
        .content(move |_, _, _| {
            div()
                .text_sm()
                .line_clamp(MESSAGE_LINES)
                .child(message.clone())
                .into_any_element()
        })
}

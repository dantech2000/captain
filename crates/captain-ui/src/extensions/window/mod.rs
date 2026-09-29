//! The window of one extension: a GPUI title bar and toast strip, and a `gpui-wry`
//! web view with the `ddClient` bridge. macOS only. See ADR 0011.

mod bridge;
mod extension_window;
mod protocol;
mod registry;
mod webview;

pub use registry::{CAN_OPEN, close_window, open_window};

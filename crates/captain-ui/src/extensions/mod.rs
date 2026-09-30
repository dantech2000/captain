//! The Extensions page: install Docker Desktop extensions by image reference, list,
//! open, update, and remove them. Each extension opens in its own window with a web view,
//! on macOS. See docs/features/0025-extensions.md and ADR 0011.

mod extension_event;
mod extension_row;
mod extensions_model;
mod extensions_view;
mod install_dialog;
mod main_window;
mod remove_dialog;
mod update_dialog;
#[cfg(target_os = "macos")]
mod window;

pub use extension_event::ExtensionEvent;
pub use extensions_model::ExtensionsModel;
pub use extensions_view::ExtensionsView;
use main_window::MainWindow;

#[cfg(not(target_os = "macos"))]
mod unsupported;

#[cfg(not(target_os = "macos"))]
use unsupported::{CAN_OPEN, close_all_windows, close_window, open_window};
#[cfg(target_os = "macos")]
use window::{CAN_OPEN, close_all_windows, close_window, open_window};

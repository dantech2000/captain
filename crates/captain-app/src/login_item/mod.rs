//! Captain's login item: a LaunchAgent on macOS, an XDG autostart entry on Linux,
//! and a `Run` key value on Windows. The text is in
//! `captain_core::behavior::login_item`. See feature 0015.

#[cfg(not(target_os = "windows"))]
mod file;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use linux::{is_enabled, set};
#[cfg(target_os = "macos")]
pub use macos::{is_enabled, set};
#[cfg(target_os = "windows")]
pub use windows::{is_enabled, set};

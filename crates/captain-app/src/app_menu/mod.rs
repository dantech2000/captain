//! The macOS menu bar: Captain, File, Edit, View, Window, and Help. GPUI builds
//! app menus only on macOS. See docs/features/0042-app-menus.md.

mod entries;
mod handlers;

pub use handlers::register;

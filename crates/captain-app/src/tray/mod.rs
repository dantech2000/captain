//! The menu bar (macOS) and notification area (Windows) icon. See ADR 0006.

mod controller;
mod events;
mod icon;
mod menu;
mod menu_model;
mod snapshot;

pub use controller::{is_running, start};

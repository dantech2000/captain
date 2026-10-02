//! The menu bar (macOS) and notification area (Windows) icon, and its native menu.
//! See ADR 0006.

mod containers_menu;
mod contexts;
mod controller;
mod dot;
mod entries;
mod events;
mod exit_facts;
mod follow;
mod gather;
mod icon;
mod icon_view;
mod kubernetes_menu;
mod look;
mod menu;
mod menu_model;
mod problem_menu;
mod projects_menu;
mod raise;
mod snapshot;
#[cfg(target_os = "macos")]
mod status_dot;
#[cfg(test)]
mod test_support;

pub use controller::is_running;
pub use follow::follow_settings;

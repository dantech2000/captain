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
mod kubernetes_menu;
mod menu;
mod menu_model;
mod problem_menu;
mod projects_menu;
mod raise;
mod snapshot;
#[cfg(test)]
mod test_support;

pub use controller::is_running;
pub use follow::follow_settings;

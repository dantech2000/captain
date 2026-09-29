//! GPUI views for Captain. Views reach the engine only through [`captain_core::Engine`].

mod containers;
mod inspector;
mod shell;
mod theme;
mod widgets;
mod workspace;

pub use shell::AppShell;
pub use workspace::Connector;

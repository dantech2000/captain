//! GPUI views for Captain. Views reach the engine only through [`captain_core::Engine`].

mod containers;
mod shell;

pub use shell::{AppShell, Connector};

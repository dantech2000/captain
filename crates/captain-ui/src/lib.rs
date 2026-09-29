//! GPUI views for Captain. Views reach the engine only through [`captain_core::Engine`].

mod containers;
mod images;
mod inspector;
mod networks;
mod palette;
mod settings;
mod shell;
mod theme;
mod volumes;
mod widgets;
mod workspace;

pub use palette::{ToggleCommandPalette, init as palette_init};
pub use settings::{
    DetectedEndpoint, EngineSource, init as settings_init, init_engine_source as engine_source_init,
};
pub use shell::AppShell;
pub use workspace::{Connection, Connector, Page, Workspace};

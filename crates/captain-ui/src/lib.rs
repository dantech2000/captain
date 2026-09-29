//! GPUI views for Captain. Views reach the engine only through [`captain_core::Engine`].

mod containers;
mod engine_host;
mod images;
mod inspector;
mod migration;
mod networks;
mod palette;
mod settings;
mod shell;
mod theme;
mod volumes;
mod widgets;
mod workspace;

pub use engine_host::{
    HostEvent, HostModel, HostSummary, host_model, init as engine_host_init,
    summary as host_summary, uses_captain,
};
pub use migration::OpenMigrationAssistant;
pub use palette::{ToggleCommandPalette, init as palette_init};
pub use settings::{
    DetectedEndpoint, EngineSource, current as current_settings, init as settings_init,
    init_engine_source as engine_source_init,
};
pub use shell::AppShell;
pub use workspace::{Connection, Connector, Page, Workspace};

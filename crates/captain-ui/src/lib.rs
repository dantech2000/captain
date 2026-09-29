//! GPUI views for Captain. Views reach the engine only through [`captain_core::Engine`].

mod containers;
mod diagnostics;
mod engine_host;
mod extensions;
mod images;
mod inspector;
mod kubernetes;
mod migration;
mod networks;
mod palette;
mod port_forwarding;
mod settings;
mod shell;
mod snapshots;
mod theme;
mod volumes;
mod widgets;
mod workspace;

pub use diagnostics::{DiagnosticsSetup, init as diagnostics_init};
pub use engine_host::{
    HostEvent, HostModel, HostSummary, host_model, init as engine_host_init,
    summary as host_summary, uses_captain,
};
pub use kubernetes::init as kubernetes_init;
pub use migration::OpenMigrationAssistant;
pub use palette::{ToggleCommandPalette, init as palette_init};
pub use port_forwarding::init as port_forwarding_init;
pub use settings::{
    DetectedEndpoint, EngineSource, SystemIntegration, current as current_settings,
    init as settings_init, init_engine_source as engine_source_init, init_system as system_init,
    observe as observe_settings,
};
pub use shell::AppShell;
pub use workspace::{Connection, Connector, Page, Workspace};

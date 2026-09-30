//! GPUI views for Captain. Views reach the engine only through [`captain_core::Engine`].

mod agents;
mod containers;
mod diagnostics;
mod engine_host;
mod extensions;
mod help;
mod icons;
mod images;
mod inspector;
mod kubernetes;
mod menu_bar;
mod migration;
mod networks;
mod palette;
mod port_forwarding;
mod project;
mod settings;
mod shell;
mod snapshots;
mod storage;
mod theme;
mod volumes;
mod widgets;
mod workspace;

pub use agents::init as agents_init;
pub use diagnostics::{
    DiagnosticsSetup, diagnostics_model, init as diagnostics_init, run_suggested_fix,
};
pub use engine_host::{
    HostEvent, HostModel, HostSummary, host_model, init as engine_host_init,
    summary as host_summary, uses_captain,
};
pub use icons::{CaptainAssets, CaptainIcon, cap_icon};
pub use kubernetes::{init as kubernetes_init, kubernetes_model};
pub use menu_bar::{observe_problem_count, open_float_log};
pub use migration::OpenMigrationAssistant;
pub use palette::{ToggleCommandPalette, init as palette_init};
pub use port_forwarding::init as port_forwarding_init;
pub use settings::{
    ContextJob, DetectedEndpoint, EngineSource, SystemIntegration, current as current_settings,
    init as settings_init, init_engine_source as engine_source_init,
    init_file_watch as settings_watch_init, init_system as system_init,
    observe as observe_settings, open_settings_file, settings_file_path,
};
pub use shell::{AppShell, engine_name};
pub use storage::init as storage_init;
pub use workspace::{Connection, Connector, Page, Workspace};

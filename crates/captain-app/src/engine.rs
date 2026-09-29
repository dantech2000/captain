//! Builds Captain Engine for this platform and decides, at launch, whether Captain
//! connects to another engine or waits for its own. See ADR 0008.

use std::sync::Arc;

use captain_core::settings::{EngineChoice, Settings};
use captain_core::{EngineHost, HostResources};
use captain_host::machine;
use captain_ui::Workspace;
use gpui_kit::*;

/// Captain Engine, and what the UI needs to know about this computer.
pub struct EngineSetup {
    host: Arc<dyn EngineHost>,
    available: bool,
    machine: HostResources,
    choice: EngineChoice,
}

impl EngineSetup {
    pub fn new(settings: &Settings) -> Self {
        let available = captain_host::captain_engine_available();
        let resources = settings
            .engine_resources
            .unwrap_or_else(machine::recommended_resources);
        let choice = settings.engine_choice(available);
        tracing::info!(?choice, available, "engine choice");
        Self {
            host: captain_host::default_host(resources),
            available,
            machine: HostResources {
                cpus: machine::host_cpus(),
                memory_bytes: machine::host_memory(),
                disk_bytes: 0,
            },
            choice,
        }
    }

    /// True if Captain connects to another engine at launch. With Captain Engine,
    /// the host model connects once the engine runs, and starts it if it is stopped.
    pub fn connects_elsewhere(&self) -> bool {
        self.choice == EngineChoice::External
    }

    /// Installs the host model for `workspace`.
    pub fn install(self, workspace: &Entity<Workspace>, cx: &mut App) {
        captain_ui::engine_host_init(cx, self.host, self.available, self.machine, workspace);
    }
}

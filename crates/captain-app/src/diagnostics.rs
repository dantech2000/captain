//! Gathers the facts that the Diagnostics page checks. See
//! docs/features/0016-diagnostics.md.

use std::path::Path;
use std::sync::Arc;

use captain_core::diagnostics::{MachineFacts, ToolProbe};
use captain_host::{LimaPaths, probe};
use captain_ui::DiagnosticsSetup;

use crate::logging::Logging;

/// The probe, the folders, and the debug switch for the Diagnostics page.
pub fn setup(logging: &Logging) -> DiagnosticsSetup {
    let home = std::env::home_dir();
    let engine_dir = home
        .as_deref()
        .filter(|_| cfg!(target_os = "macos"))
        .map(|home| LimaPaths::for_home(home).instance_dir());
    let probe_dir = engine_dir.clone();
    DiagnosticsSetup {
        probe: Arc::new(move || machine_facts(home.as_deref(), probe_dir.as_deref())),
        log_dir: logging.dir.clone(),
        engine_dir,
        set_debug_logging: logging.debug_switch(),
    }
}

/// Runs the probes side by side. Each command has its own time limit.
fn machine_facts(home: Option<&Path>, engine_dir: Option<&Path>) -> MachineFacts {
    std::thread::scope(|scope| {
        let docker = scope.spawn(captain_docker::docker_tools);
        let lima = scope.spawn(|| {
            if cfg!(target_os = "macos") {
                probe::lima_version()
            } else {
                ToolProbe::Missing
            }
        });
        let free_disk = home.and_then(probe::free_space);
        let lima_logs = engine_dir.and_then(probe::log_bytes);
        let broken = || ToolProbe::Broken("The check stopped.".into());
        let (docker, compose) = docker.join().unwrap_or_else(|_| (broken(), broken()));
        MachineFacts {
            lima: lima.join().unwrap_or_else(|_| broken()),
            docker,
            compose,
            free_disk,
            lima_logs,
            rosetta: probe::rosetta_installed(),
        }
    })
}

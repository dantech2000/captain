//! Applies the Docker daemon settings after a start, and learns what a running
//! engine uses, for "Restart to apply". See feature 0020.

use captain_core::daemon::{DaemonSettings, DaemonState};
use captain_core::{HostError, HostStatus};

use super::{Inner, lock, steps};
use crate::LimaHost;
use crate::lima::daemon;
use crate::lima::limactl::Limactl;

/// Installs the saved settings in the running VM and restarts Docker, unless the VM
/// has them already. On a failure the engine keeps the settings it had.
pub fn apply(
    inner: &Inner,
    limactl: &Limactl,
    sink: &mut dyn FnMut(String),
) -> Result<(), HostError> {
    let wanted = lock(&inner.daemon).state();
    let current = read(inner, limactl);
    if current.as_ref() != Some(&wanted) {
        sink("Applying the Docker daemon settings.".into());
        let args = daemon::write_args(&inner.paths.instance, &wanted);
        if let Err(error) = limactl.run(&args, Some(&daemon::write_input(&wanted)), &inner.cancel) {
            *lock(&inner.running_daemon) = current;
            return Err(HostError(format!(
                "Docker did not accept the daemon settings, so Captain Engine keeps the previous ones. {error}"
            )));
        }
    }
    *lock(&inner.running_daemon) = Some(wanted);
    Ok(())
}

/// Forgets the running settings when the engine is not running, and reads them once
/// when it runs and they are not known, for example after Captain launched.
pub fn track(inner: &Inner, limactl: &Limactl, status: &HostStatus) {
    if *status != HostStatus::Running {
        *lock(&inner.running_daemon) = None;
        return;
    }
    if lock(&inner.running_daemon).is_some() {
        return;
    }
    if let Some(state) = read(inner, limactl) {
        *lock(&inner.running_daemon) = Some(state);
    }
}

fn read(inner: &Inner, limactl: &Limactl) -> Option<DaemonState> {
    let args = daemon::read_args(&inner.paths.instance);
    let output = limactl.output_within(&args, steps::QUICK_TIMEOUT);
    daemon::parse_state(&output.ok()?)
}

impl LimaHost {
    /// The daemon settings for the next start, for a snapshot's metadata.
    pub(crate) fn daemon_settings(&self) -> DaemonSettings {
        lock(&self.inner.daemon).clone()
    }
}

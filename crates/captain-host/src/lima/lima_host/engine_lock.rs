//! The lock that keeps two Captain processes, such as the app and the `captain` CLI,
//! from starting or stopping the engine at the same time. Lima has no lock of its
//! own for this. See docs/features/0022-command-line.md.

use captain_core::process_lock::ProcessLock;
use captain_core::{HostError, HostStatus};

use super::Inner;

const STARTING: &str = "starting";
const STOPPING: &str = "stopping";
/// A resource change on a stopped instance.
pub const EDITING: &str = "editing";
/// A snapshot create, restore, or delete.
pub const SNAPSHOT: &str = "snapshot";

/// Takes the lock for a start (`starting`) or a stop or reset (`stopping`), or
/// fails when another process holds it.
pub fn acquire(inner: &Inner, starting: bool) -> Result<ProcessLock, HostError> {
    let note = if starting { STARTING } else { STOPPING };
    acquire_with(inner, note)
}

/// Takes the lock with `note`, or fails when another process holds it.
pub fn acquire_with(inner: &Inner, note: &str) -> Result<ProcessLock, HostError> {
    try_acquire(inner, note)?.ok_or_else(|| busy(inner))
}

/// Takes the lock with `note`, or `None` when another process holds it.
pub fn try_acquire(inner: &Inner, note: &str) -> Result<Option<ProcessLock>, HostError> {
    let path = inner.paths.lock_file();
    ProcessLock::try_acquire(&path, note)
        .map_err(|error| HostError(format!("Cannot lock {}: {error}", path.display())))
}

/// Starting or Stopping while another process starts or stops the engine.
pub fn other_process(inner: &Inner) -> Option<HostStatus> {
    match ProcessLock::holder(&inner.paths.lock_file())?.as_str() {
        STARTING => Some(HostStatus::Starting),
        STOPPING => Some(HostStatus::Stopping),
        _ => None,
    }
}

fn busy(inner: &Inner) -> HostError {
    let what = match other_process(inner) {
        Some(HostStatus::Starting) => "starting",
        Some(HostStatus::Stopping) => "stopping",
        _ if ProcessLock::holder(&inner.paths.lock_file()).as_deref() == Some(SNAPSHOT) => {
            "busy with a snapshot"
        }
        _ => "busy",
    };
    HostError(format!(
        "Captain Engine is {what} in another Captain process."
    ))
}

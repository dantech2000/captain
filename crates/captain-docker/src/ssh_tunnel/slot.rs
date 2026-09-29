//! The one open tunnel, guarded against a start that finishes after a newer one.

use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use captain_core::ssh::SshTarget;

use super::SshTunnel;

/// The open tunnel, and a count of opens and closes. A start that finishes after a
/// newer open or close stops its own tunnel instead of replacing the newer one.
pub(crate) struct Slot {
    pub(super) generation: u64,
    pub(super) tunnel: Option<SshTunnel>,
}

impl Slot {
    pub(crate) const fn new() -> Self {
        Self {
            generation: 0,
            tunnel: None,
        }
    }

    pub(super) fn lock(slot: &Mutex<Self>) -> std::sync::MutexGuard<'_, Self> {
        slot.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Opens a tunnel in `slot` with `start`, unless the open one already goes to `target`.
pub(crate) fn open_in(
    slot: &Mutex<Slot>,
    target: &SshTarget,
    start: impl FnOnce() -> Result<SshTunnel, String>,
) -> Result<PathBuf, String> {
    let (old, generation) = {
        let mut slot = Slot::lock(slot);
        if let Some(tunnel) = slot
            .tunnel
            .as_ref()
            .filter(|tunnel| tunnel.target == *target)
        {
            return Ok(tunnel.socket.clone());
        }
        slot.generation += 1;
        (slot.tunnel.take(), slot.generation)
    };
    drop(old);
    let tunnel = start()?;
    let socket = tunnel.socket.clone();
    let mut current = Slot::lock(slot);
    if current.generation != generation {
        drop(current);
        drop(tunnel);
        return Err(format!(
            "The connection to {target} was replaced by a newer one."
        ));
    }
    let replaced = current.tunnel.replace(tunnel);
    drop(current);
    drop(replaced);
    Ok(socket)
}

//! The one open tunnel, guarded against a start that finishes after a newer one.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use captain_core::ssh::SshTarget;

use super::{Shared, SshTunnel};

/// The open tunnel, the start in progress, and a count of opens and closes. A start
/// that finishes after a newer open or close stops its own tunnel instead of
/// replacing the newer one.
pub(crate) struct Slot {
    pub(super) generation: u64,
    pub(super) tunnel: Option<SshTunnel>,
    /// The start in progress, so a close can kill its `ssh` while it logs in.
    starting: Option<Arc<Shared>>,
}

impl Slot {
    pub(crate) const fn new() -> Self {
        Self {
            generation: 0,
            tunnel: None,
            starting: None,
        }
    }

    pub(super) fn lock(slot: &Mutex<Self>) -> std::sync::MutexGuard<'_, Self> {
        slot.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Lets a start record itself in the slot it opens in.
pub(crate) struct Register<'a> {
    slot: &'a Mutex<Slot>,
    generation: u64,
}

impl Register<'_> {
    /// Records `shared` as the start in progress. False if a newer open or close
    /// came first, so the start must not run.
    pub(super) fn starting(&self, shared: &Arc<Shared>) -> bool {
        let mut slot = Slot::lock(self.slot);
        if slot.generation != self.generation {
            return false;
        }
        slot.starting = Some(shared.clone());
        true
    }
}

/// Opens a tunnel in `slot` with `start`, unless the open one already goes to `target`.
pub(crate) fn open_in(
    slot: &Mutex<Slot>,
    target: &SshTarget,
    start: impl FnOnce(Option<Register<'_>>) -> Result<SshTunnel, String>,
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
    let started = start(Some(Register { slot, generation }));
    let mut current = Slot::lock(slot);
    if current.generation == generation {
        current.starting = None;
    }
    let tunnel = started?;
    let socket = tunnel.socket.clone();
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

/// Stops the open tunnel and the start in progress in `slot`.
pub(crate) fn close_in(slot: &Mutex<Slot>) {
    let (tunnel, starting) = {
        let mut slot = Slot::lock(slot);
        slot.generation += 1;
        (slot.tunnel.take(), slot.starting.take())
    };
    if let Some(starting) = starting {
        starting.stop();
    }
    drop(tunnel);
}

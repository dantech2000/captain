//! [`LimaSnapshots`]: the [`EngineSnapshots`] of a [`LimaHost`].

use captain_core::HostFuture;
use captain_core::snapshot::{EngineSnapshots, Snapshot, SnapshotList};

use super::steps;
use crate::LimaHost;
use crate::blocking::blocking;

/// Snapshots of one Lima instance, in the `snapshots` folder next to its `LIMA_HOME`.
pub struct LimaSnapshots {
    host: LimaHost,
}

impl LimaSnapshots {
    pub fn new(host: LimaHost) -> Self {
        Self { host }
    }
}

impl EngineSnapshots for LimaSnapshots {
    fn list(&self) -> HostFuture<SnapshotList> {
        let host = self.host.clone();
        blocking(move || Ok(steps::list(&host)))
    }

    fn create(&self, name: String, description: String) -> HostFuture<Snapshot> {
        let host = self.host.clone();
        blocking(move || steps::create(&host, &name, &description))
    }

    fn restore(&self, id: String) -> HostFuture<Snapshot> {
        let host = self.host.clone();
        blocking(move || steps::restore(&host, &id))
    }

    fn delete(&self, id: String) -> HostFuture<()> {
        let host = self.host.clone();
        blocking(move || steps::delete(&host, &id))
    }
}

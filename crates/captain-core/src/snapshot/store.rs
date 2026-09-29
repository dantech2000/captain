use crate::HostFuture;

use super::Snapshot;

/// The complete snapshots, newest first, the free bytes on their disk, and the bytes
/// the engine's disk uses now.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SnapshotList {
    pub snapshots: Vec<Snapshot>,
    pub free_bytes: Option<u64>,
    /// `None` before the engine is set up.
    pub engine_disk_bytes: Option<u64>,
}

/// Makes, lists, restores, and deletes snapshots of a stopped engine. Create and
/// restore fail while the engine runs; the caller stops it first. Like
/// [`crate::EngineHost`], the futures must not depend on a specific async runtime.
pub trait EngineSnapshots: Send + Sync + 'static {
    fn list(&self) -> HostFuture<SnapshotList>;

    /// Saves the stopped engine as a new snapshot.
    fn create(&self, name: String, description: String) -> HostFuture<Snapshot>;

    /// Replaces the stopped engine with the snapshot `id`. It returns the snapshot, so
    /// the caller can save its resources in the settings.
    fn restore(&self, id: String) -> HostFuture<Snapshot>;

    fn delete(&self, id: String) -> HostFuture<()>;

    /// Renames the snapshot `id` and sets its description. The engine may run.
    fn edit(&self, id: String, name: String, description: String) -> HostFuture<Snapshot>;
}

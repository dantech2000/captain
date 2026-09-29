use std::sync::Arc;
use std::time::Duration;

use super::{MigrationItem, MigrationPlan, SwitchOverStep};
use crate::{EngineError, EngineFuture, EngineStream};

/// An engine the user can copy from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceOption {
    /// Where Captain found it, for example "Socket" or "Current context".
    pub label: String,
    /// The `DOCKER_HOST`-style URL.
    pub host: String,
}

/// What a copy reports while it runs. The stream ends when the item is copied, and
/// fails with the reason when it is not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferEvent {
    /// Bytes copied so far. `total` is 0 when the size is unknown.
    Progress { done: u64, total: u64 },
    /// Something to show next to the finished item.
    Note(String),
    /// The item was not copied, for the reason given.
    Skipped(String),
    /// A switch-over entered this step.
    SwitchOver(SwitchOverStep),
    /// How long the item was down during a switch-over: from the stop in the source
    /// until the check in the target passed.
    Downtime(Duration),
}

/// Finds engines and opens migration sessions. The app implements it, because only
/// the app knows the engine crate. See docs/adr/0009-migration.md.
pub trait MigrationBackend: Send + Sync + 'static {
    /// Engines found on this machine, without `target`.
    fn sources(&self, target: &str) -> Vec<SourceOption>;

    /// Connects to both engines. It blocks for up to a few seconds, so call it from
    /// a background thread.
    fn open(&self, source: &str, target: &str) -> Result<Arc<dyn MigrationSession>, EngineError>;
}

/// A connection to a source and a target engine.
///
/// The session never changes the source engine. It only lists, inspects, and exports
/// from it, and runs its own helper containers there, named `captain-migrate-*`,
/// which it removes again. There are two exceptions, both opt-in. A snapshot commits
/// a container to a `captain-migrate/<name>:snapshot` image, copies it, and removes
/// it. A switch-over stops the containers the user confirmed and never removes them.
pub trait MigrationSession: Send + Sync + 'static {
    /// The source endpoint.
    fn source(&self) -> &str;

    /// Lists what the source engine holds.
    fn scan(&self) -> EngineFuture<MigrationPlan>;

    /// Free bytes on the target engine's disk, or `None` if it cannot tell.
    fn target_free_space(&self) -> EngineFuture<Option<u64>>;

    /// Copies one item into the target. Dropping the stream cancels the copy and
    /// removes its helpers and any half-copied volume.
    fn copy(&self, item: &MigrationItem, snapshot: bool) -> EngineStream<TransferEvent>;

    /// Switches `item` over: stops its running containers in the source (never
    /// removes them), copies its volumes again, starts it in the target, and checks
    /// it. It reports each step with [`TransferEvent::SwitchOver`] and ends with
    /// [`TransferEvent::Downtime`]. Once the source is stopped, dropping the stream
    /// no longer stops the switch-over, so the item is not left half-moved.
    fn switch_over(&self, item: &MigrationItem, snapshot: bool) -> EngineStream<TransferEvent>;

    /// Undoes a switch-over of `item`: stops its containers in the target and starts
    /// the stopped originals in the source. Nothing is removed on either side.
    fn roll_back(&self, item: &MigrationItem) -> EngineFuture<()>;

    /// Waits for stopped copies to clean up, then removes helper containers that
    /// are left and a helper image the session pulled into the source. Keep the
    /// session alive until the future ends, because the session runs the cleanup.
    fn finish(&self) -> EngineFuture<()>;
}

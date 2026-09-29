//! What a snapshot step needs from [`HostModel`]: block the engine controls, stop
//! the engine, adopt a restored snapshot's settings, and start it again. See
//! docs/features/0023-snapshots.md.

use std::sync::Arc;

use captain_core::HostFuture;
use captain_core::snapshot::{EngineSnapshots, SnapshotMetadata};
use gpui_kit::*;

use super::HostModel;
use crate::settings;

impl HostModel {
    /// The snapshots of Captain Engine, or `None` where it has none.
    pub fn snapshots(&self) -> Option<Arc<dyn EngineSnapshots>> {
        self.host.snapshots()
    }

    pub fn is_snapshotting(&self) -> bool {
        self.snapshotting
    }

    /// Blocks the engine controls and stops the engine if it runs. Returns whether it
    /// ran, and a task that ends when it has stopped.
    pub(crate) fn begin_snapshot(&mut self, cx: &mut Context<Self>) -> (bool, Task<()>) {
        let running = self.status.is_running() || self.start_task.is_some();
        let stop = if running {
            self.stop_now(cx)
        } else {
            Task::ready(())
        };
        self.snapshotting = true;
        cx.notify();
        (running, stop)
    }

    /// Saves the resources, daemon, and Kubernetes settings of a restored snapshot,
    /// so the next start does not change the restored engine.
    pub(crate) fn adopt_snapshot(
        &mut self,
        metadata: &SnapshotMetadata,
        cx: &mut Context<Self>,
    ) -> HostFuture<()> {
        settings::update(cx, |settings| metadata.adopt_into(settings));
        let saved = settings::current(cx);
        self.host.set_daemon(saved.engine_daemon);
        self.host.set_kubernetes(saved.kubernetes);
        cx.notify();
        self.host.set_resources(metadata.resources)
    }

    /// Keeps the running snapshot step from starting the engine again, because
    /// Captain quits when it ends.
    pub fn quit_after_snapshot(&mut self, cx: &mut Context<Self>) {
        self.quitting = true;
        cx.notify();
    }

    /// Unblocks the engine controls, and starts the engine if `restart`, unless
    /// Captain quits.
    pub(crate) fn end_snapshot(&mut self, restart: bool, cx: &mut Context<Self>) {
        self.snapshotting = false;
        if restart && !self.quitting {
            self.start(cx);
        }
        cx.notify();
    }
}

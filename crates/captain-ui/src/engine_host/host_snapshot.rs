//! What a snapshot step needs from [`HostModel`]: block the engine controls, stop
//! the engine, adopt restored resources, and start it again. See
//! docs/features/0023-snapshots.md.

use std::sync::Arc;

use captain_core::snapshot::EngineSnapshots;
use captain_core::{HostFuture, HostResources};
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

    /// Saves the resources of a restored snapshot, so the next start does not edit
    /// its `lima.yaml` back.
    pub(crate) fn adopt_resources(
        &mut self,
        resources: HostResources,
        cx: &mut Context<Self>,
    ) -> HostFuture<()> {
        settings::update(cx, |settings| settings.engine_resources = Some(resources));
        self.host.set_resources(resources)
    }

    /// Unblocks the engine controls, and starts the engine if `restart`.
    pub(crate) fn end_snapshot(&mut self, restart: bool, cx: &mut Context<Self>) {
        self.snapshotting = false;
        if restart {
            self.start(cx);
        }
        cx.notify();
    }
}

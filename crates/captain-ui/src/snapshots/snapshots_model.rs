use std::sync::Arc;

use captain_core::snapshot::{EngineSnapshots, SnapshotList};
use gpui_kit::*;

use super::SnapshotEvent;
use crate::engine_host::{HostModel, host_model};

/// The snapshot list and the step that runs now. The steps live in
/// `snapshot_steps.rs`.
pub struct SnapshotsModel {
    pub(super) host: Option<Entity<HostModel>>,
    pub(super) store: Option<Arc<dyn EngineSnapshots>>,
    pub(super) list: SnapshotList,
    pub(super) loaded: bool,
    /// What the running step does now, for the page header.
    pub(super) step: Option<SharedString>,
    pub(super) task: Option<Task<()>>,
    load: Option<Task<()>>,
}

impl EventEmitter<SnapshotEvent> for SnapshotsModel {}

impl SnapshotsModel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let host = host_model(cx);
        let store = host.as_ref().and_then(|host| host.read(cx).snapshots());
        let mut model = Self {
            host,
            store,
            list: SnapshotList::default(),
            loaded: false,
            step: None,
            task: None,
            load: None,
        };
        model.reload(cx);
        model
    }

    pub fn list(&self) -> &SnapshotList {
        &self.list
    }

    pub fn is_available(&self) -> bool {
        self.store.is_some()
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// True while a create, restore, or delete runs.
    pub fn is_busy(&self) -> bool {
        self.task.is_some()
    }

    pub fn step(&self) -> Option<SharedString> {
        self.step.clone()
    }

    /// Reads the snapshots folder again.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(store) = self.store.clone() else {
            return;
        };
        let list = store.list();
        self.load = Some(cx.spawn(async move |this, cx| {
            let result = list.await;
            this.update(cx, |model, cx| {
                model.load = None;
                model.loaded = true;
                match result {
                    Ok(list) => model.list = list,
                    Err(error) => cx.emit(SnapshotEvent::Failed {
                        action: "List",
                        message: error.0,
                    }),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    pub(super) fn set_step(&mut self, step: &'static str, cx: &mut Context<Self>) {
        self.step = Some(step.into());
        cx.notify();
    }
}

//! Create, restore, and delete. Create and restore stop Captain Engine first, block
//! its controls meanwhile, and start it again if it ran.

use captain_core::HostError;
use captain_core::snapshot::Snapshot;
use gpui_kit::*;

use super::{SnapshotEvent, SnapshotsModel};

impl SnapshotsModel {
    /// Saves the engine as a new snapshot.
    pub fn create(&mut self, name: String, description: String, cx: &mut Context<Self>) {
        let (Some(host), Some(store)) = (self.host.clone(), self.store.clone()) else {
            return;
        };
        if self.task.is_some() {
            return;
        }
        let (running, stop) = host.update(cx, |host, cx| host.begin_snapshot(cx));
        self.set_step(stopping_or(running, "Saving the snapshot..."), cx);
        self.task = Some(cx.spawn(async move |this, cx| {
            stop.await;
            this.update(cx, |model, cx| model.set_step("Saving the snapshot...", cx))
                .ok();
            let result = store.create(name, description).await;
            let done = result.map(|snapshot| format!("Saved \"{}\"", snapshot.metadata.name));
            host.update(cx, |host, cx| host.end_snapshot(running, cx));
            this.update(cx, |model, cx| model.finish("Create", done, cx))
                .ok();
        }));
    }

    /// Replaces the engine with the snapshot `id`. With `save_first`, the current
    /// state becomes a snapshot first.
    pub fn restore(&mut self, snapshot: Snapshot, save_first: bool, cx: &mut Context<Self>) {
        let (Some(host), Some(store)) = (self.host.clone(), self.store.clone()) else {
            return;
        };
        if self.task.is_some() {
            return;
        }
        let (running, stop) = host.update(cx, |host, cx| host.begin_snapshot(cx));
        self.set_step(stopping_or(running, "Restoring the snapshot..."), cx);
        self.task = Some(cx.spawn(async move |this, cx| {
            stop.await;
            let name = snapshot.metadata.name.clone();
            let mut result = Ok(());
            if save_first {
                this.update(cx, |model, cx| {
                    model.set_step("Saving the current state...", cx)
                })
                .ok();
                let backup = format!("Before restore {}", now());
                let description = format!("Saved before restoring \"{name}\".");
                result = store.create(backup, description).await.map(drop);
            }
            if result.is_ok() {
                this.update(cx, |model, cx| {
                    model.set_step("Restoring the snapshot...", cx)
                })
                .ok();
                result = match store.restore(snapshot.id).await {
                    Ok(restored) => {
                        let resources = restored.metadata.resources;
                        host.update(cx, |host, cx| host.adopt_resources(resources, cx))
                            .await
                    }
                    Err(error) => Err(error),
                };
            }
            host.update(cx, |host, cx| host.end_snapshot(running, cx));
            let done = result.map(|()| format!("Restored \"{name}\""));
            this.update(cx, |model, cx| model.finish("Restore", done, cx))
                .ok();
        }));
    }

    /// Deletes the snapshot. The engine keeps running.
    pub fn delete(&mut self, snapshot: Snapshot, cx: &mut Context<Self>) {
        let Some(store) = self.store.clone() else {
            return;
        };
        if self.task.is_some() {
            return;
        }
        self.set_step("Deleting the snapshot...", cx);
        let delete = store.delete(snapshot.id);
        let name = snapshot.metadata.name;
        self.task = Some(cx.spawn(async move |this, cx| {
            let done = delete.await.map(|()| format!("Deleted \"{name}\""));
            this.update(cx, |model, cx| model.finish("Delete", done, cx))
                .ok();
        }));
    }

    fn finish(
        &mut self,
        action: &'static str,
        result: Result<String, HostError>,
        cx: &mut Context<Self>,
    ) {
        self.task = None;
        self.step = None;
        cx.emit(match result {
            Ok(message) => SnapshotEvent::Done(message),
            Err(error) => SnapshotEvent::Failed {
                action,
                message: error.0,
            },
        });
        self.reload(cx);
        cx.notify();
    }
}

fn stopping_or(running: bool, step: &'static str) -> &'static str {
    if running {
        "Stopping Captain Engine..."
    } else {
        step
    }
}

/// The local date and time, for the name of the snapshot saved before a restore.
fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

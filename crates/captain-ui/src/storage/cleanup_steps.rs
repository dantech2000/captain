//! The cleanup: an optional snapshot of Captain Engine first, then the removals. The
//! snapshot step stops the engine and starts it again, like the Snapshots page's
//! Create; the removals wait until the workspace has connected to it again. The task
//! is detached, so closing the window does not cut it off.

use std::sync::Arc;
use std::time::Duration;

use captain_core::Engine;
use captain_core::storage::{ReclaimItem, run_cleanup};
use gpui_kit::*;

use super::{StorageEvent, StorageModel};
use crate::engine_host::{host_model, uses_captain};
use crate::workspace::{Connection, Workspace};

/// How long the cleanup waits for Captain Engine to come back after the snapshot.
const RECONNECT_WAIT: Duration = Duration::from_secs(3 * 60);
const RECONNECT_POLL: Duration = Duration::from_millis(500);

impl StorageModel {
    /// True if the snapshot option applies: Captain Engine with snapshots.
    pub fn can_snapshot(&self, cx: &App) -> bool {
        uses_captain(cx)
            && host_model(cx).is_some_and(|host| {
                let host = host.read(cx);
                host.snapshots().is_some() && !host.is_snapshotting()
            })
    }

    pub fn step(&self) -> Option<SharedString> {
        self.step.clone()
    }

    /// Removes `items`, the ones the preview listed, after a snapshot if that option
    /// is on.
    pub fn clean_up(&mut self, items: Vec<ReclaimItem>, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        if items.is_empty() || self.step.is_some() {
            return;
        }
        let snapshot = (self.snapshot_first && self.can_snapshot(cx))
            .then(|| host_model(cx))
            .flatten()
            .and_then(|host| {
                let store = host.read(cx).snapshots()?;
                Some((host, store))
            });
        let workspace = self.workspace.clone();
        self.set_step("Removing...", cx);
        cx.spawn(async move |this, cx| {
            let engine = match snapshot {
                None => engine,
                Some((host, store)) => {
                    this.update(cx, |model, cx| model.set_step("Taking a snapshot...", cx))
                        .ok();
                    let (running, stop) = host.update(cx, |host, cx| host.begin_snapshot(cx));
                    stop.await;
                    let name = format!("Before cleanup {}", local_now());
                    let description =
                        format!("Saved before Storage removed {} items.", items.len());
                    let saved = store.create(name, description).await;
                    host.update(cx, |host, cx| host.end_snapshot(running, cx));
                    if let Err(error) = saved {
                        let message =
                            format!("The snapshot failed, so nothing was removed: {error}");
                        this.update(cx, |model, cx| model.finish(Err(message), cx))
                            .ok();
                        return;
                    }
                    this.update(cx, |model, cx| {
                        model.set_step("Waiting for Captain Engine...", cx)
                    })
                    .ok();
                    match reconnected(&workspace, &engine, cx).await {
                        Some(engine) => engine,
                        None => {
                            let message = "Captain Engine did not come back after the \
                                           snapshot, so nothing was removed."
                                .to_string();
                            this.update(cx, |model, cx| model.finish(Err(message), cx))
                                .ok();
                            return;
                        }
                    }
                }
            };
            this.update(cx, |model, cx| model.set_step("Removing...", cx))
                .ok();
            let report = run_cleanup(engine, items).await;
            let result = if report.failures.is_empty() {
                Ok(report.summary())
            } else {
                Err(report.summary())
            };
            this.update(cx, |model, cx| model.finish(result, cx)).ok();
        })
        .detach();
    }

    fn set_step(&mut self, step: &'static str, cx: &mut Context<Self>) {
        self.step = Some(step.into());
        cx.notify();
    }

    fn finish(&mut self, result: Result<String, String>, cx: &mut Context<Self>) {
        self.step = None;
        cx.emit(match result {
            Ok(message) => StorageEvent::Done(message),
            Err(message) => StorageEvent::Failed(message),
        });
        self.reload(cx);
    }
}

/// The workspace's engine once it is connected to a new one, or `None` after
/// [`RECONNECT_WAIT`].
async fn reconnected(
    workspace: &WeakEntity<Workspace>,
    old: &Arc<dyn Engine>,
    cx: &mut AsyncApp,
) -> Option<Arc<dyn Engine>> {
    let tries = RECONNECT_WAIT.as_millis() / RECONNECT_POLL.as_millis();
    for _ in 0..tries {
        cx.background_executor().timer(RECONNECT_POLL).await;
        let engine = workspace
            .read_with(cx, |workspace, _| {
                let connected = matches!(workspace.connection(), Connection::Connected(_));
                workspace.engine().filter(|_| connected)
            })
            .ok()?;
        if let Some(engine) = engine.filter(|engine| !Arc::ptr_eq(engine, old)) {
            return Some(engine);
        }
    }
    None
}

/// The local date and time, for the snapshot's name.
fn local_now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

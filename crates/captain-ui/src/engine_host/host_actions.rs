//! Start, stop, restart, reset, resize, and status polling for [`HostModel`].

use std::time::Duration;

use captain_core::settings::EngineChoice;
use captain_core::{HostError, HostResources, HostStatus};
use futures::StreamExt;
use gpui_kit::*;

use super::host_reload::report_resize;
use super::{HostEvent, HostModel};
use crate::settings;

/// How often to check the engine while it is not running.
const POLL_IDLE: Duration = Duration::from_secs(3);
/// How often to check it while it runs.
const POLL_RUNNING: Duration = Duration::from_secs(15);

impl HostModel {
    /// Checks the status now and then every few seconds, while Captain Engine is
    /// the choice.
    pub(super) fn poll(&mut self, cx: &mut Context<Self>) {
        self._poll = Some(cx.spawn(async move |this, cx| {
            loop {
                let Ok(check) = this.update(cx, |model, cx| {
                    (model.uses_captain(cx) && !model.stopping).then(|| model.host.status())
                }) else {
                    return;
                };
                if let Some(check) = check {
                    let status = check.await;
                    this.update(cx, |model, cx| model.apply_status(status, cx))
                        .ok();
                }
                let running = this
                    .read_with(cx, |model, _| model.status.is_running())
                    .unwrap_or(false);
                let delay = if running { POLL_RUNNING } else { POLL_IDLE };
                cx.background_executor().timer(delay).await;
            }
        }));
    }

    fn apply_status(&mut self, status: Result<HostStatus, HostError>, cx: &mut Context<Self>) {
        let status = status.unwrap_or_else(|error| HostStatus::Failed(error.0));
        let was_running = self.status.is_running();
        let first = std::mem::replace(&mut self.checking, false);
        if status == HostStatus::NotCreated && self.status != HostStatus::NotCreated {
            self.rescan(cx);
        }
        self.status = status;
        let idle = self.start_task.is_none() && !self.stopping;
        if idle && self.uses_captain(cx) {
            if self.status.is_running() && (!was_running || first) {
                self.connect_workspace(cx);
            } else if first && self.status == HostStatus::Stopped {
                // At launch a stopped engine starts by itself. See ADR 0008.
                self.start(cx);
            }
        }
        cx.notify();
    }

    /// Creates the engine if needed and starts it, then connects the workspace.
    pub fn start(&mut self, cx: &mut Context<Self>) {
        if self.start_task.is_some() || !self.can_control() {
            return;
        }
        let setup = self.status == HostStatus::NotCreated;
        self.status = HostStatus::Starting;
        self.log.clear();
        self.last_error = None;
        self.cancelled = false;
        let mut progress = self.host.start();
        cx.notify();
        self.start_task = Some(cx.spawn(async move |this, cx| {
            let mut failure = None;
            while let Some(line) = progress.next().await {
                match line {
                    Ok(line) => {
                        this.update(cx, |model, cx| {
                            model.log.push(line);
                            cx.notify();
                        })
                        .ok();
                    }
                    Err(error) => {
                        failure = Some(error);
                        break;
                    }
                }
            }
            this.update(cx, |model, cx| model.start_finished(failure, setup, cx))
                .ok();
        }));
    }

    fn start_finished(&mut self, failure: Option<HostError>, setup: bool, cx: &mut Context<Self>) {
        self.start_task = None;
        match failure {
            None => {
                self.status = HostStatus::Running;
                self.connect_workspace(cx);
                // Only when the setup screen listed another engine and offered the copy.
                if setup && self.migrate_after_setup && !self.detected.is_empty() {
                    cx.emit(HostEvent::OpenMigration);
                }
            }
            Some(_) if self.cancelled => {}
            Some(error) => {
                self.log.push(format!("Error: {error}"));
                self.status = HostStatus::Failed(error.0.clone());
                self.last_error = Some(error.0.clone());
                cx.emit(HostEvent::Failed {
                    action: "Start",
                    message: error.0,
                });
            }
        }
        cx.notify();
    }

    /// Stops the engine. The task ends when it has stopped. It does nothing while a
    /// snapshot step runs.
    pub fn stop(&mut self, cx: &mut Context<Self>) -> Task<()> {
        if self.snapshotting {
            return Task::ready(());
        }
        self.stop_now(cx)
    }

    pub(super) fn stop_now(&mut self, cx: &mut Context<Self>) -> Task<()> {
        self.cancelled = self.start_task.is_some();
        self.status = HostStatus::Stopping;
        self.stopping = true;
        cx.notify();
        let host = self.host.clone();
        let stop = host.stop();
        cx.spawn(async move |this, cx| {
            let result = stop.await;
            let status = host.status().await;
            this.update(cx, |model, cx| {
                model.stopping = false;
                if let Err(error) = result {
                    cx.emit(HostEvent::Failed {
                        action: "Stop",
                        message: error.0,
                    });
                }
                model.apply_status(status, cx);
            })
            .ok();
        })
    }

    /// Stops the engine, then starts it again.
    pub fn restart(&mut self, cx: &mut Context<Self>) {
        let stop = self.stop(cx);
        cx.spawn(async move |this, cx| {
            stop.await;
            this.update(cx, |model, cx| model.start(cx)).ok();
        })
        .detach();
    }

    /// Deletes the engine with all its containers, images, and volumes.
    pub fn reset(&mut self, cx: &mut Context<Self>) {
        if self.snapshotting {
            return;
        }
        self.cancelled = self.start_task.is_some();
        self.status = HostStatus::Stopping;
        self.stopping = true;
        cx.notify();
        let host = self.host.clone();
        let reset = host.reset();
        cx.spawn(async move |this, cx| {
            let result = reset.await;
            let status = host.status().await;
            this.update(cx, |model, cx| {
                model.stopping = false;
                model.log.clear();
                model.last_error = None;
                match result {
                    // The cluster went with the VM, so its context goes too (ADR 0010).
                    Ok(()) => forget_kubernetes_context(),
                    Err(error) => cx.emit(HostEvent::Failed {
                        action: "Reset",
                        message: error.0,
                    }),
                }
                model.apply_status(status, cx);
            })
            .ok();
        })
        .detach();
    }

    /// Saves new resources. A stopped engine gets them now, and a running engine on
    /// its next start; until then Settings shows "Restart to apply".
    pub fn set_resources(&mut self, resources: HostResources, cx: &mut Context<Self>) {
        if self.snapshotting {
            return;
        }
        settings::update(cx, |settings| settings.engine_resources = Some(resources));
        let apply = self.host.set_resources(resources);
        cx.spawn(async move |this, cx| {
            let result = apply.await;
            this.update(cx, |_, cx| {
                report_resize(result, cx);
                // A grown disk raises the disk stepper's floor.
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// Switches to another engine and connects to it.
    pub fn use_external(&mut self, cx: &mut Context<Self>) {
        settings::update(cx, |settings| {
            settings.engine = Some(EngineChoice::External)
        });
        self.connect_workspace(cx);
        cx.notify();
    }

    /// Switches to Captain Engine: connects if it runs, starts it if it is stopped,
    /// and otherwise leaves the setup screen up.
    pub fn use_captain(&mut self, cx: &mut Context<Self>) {
        if !self.supported() {
            return;
        }
        settings::update(cx, |settings| settings.engine = Some(EngineChoice::Captain));
        if self.checking {
            // The first status check connects or starts it.
            cx.notify();
            return;
        }
        match self.status {
            HostStatus::Running => self.connect_workspace(cx),
            HostStatus::Stopped => self.start(cx),
            _ => {}
        }
        cx.notify();
    }

    /// Reconnects the workspace to the engine the settings choose.
    /// Runs inside this model's update, so it must not call anything that reads the
    /// model again (see `settings::reconnect_to`).
    fn connect_workspace(&self, cx: &mut Context<Self>) {
        if let Some(workspace) = self.workspace.upgrade() {
            let captain = self
                .uses_captain(cx)
                .then(|| self.host.endpoint())
                .flatten();
            settings::reconnect_to(&workspace, captain, cx);
        }
    }
}

/// Removes the `captain` context from the user's kubeconfig, if it is there.
fn forget_kubernetes_context() {
    let paths = captain_core::kubernetes::user_kubeconfig_paths();
    if let Err(error) = captain_core::kubernetes::uninstall_captain(&paths) {
        tracing::warn!(%error, "cannot remove the captain-engine context");
    }
}

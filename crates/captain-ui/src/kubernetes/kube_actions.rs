//! Saving the settings, Apply, Reset, and the version list of [`KubernetesModel`].

use captain_core::HostStream;
use captain_core::kubernetes::KubernetesSettings;
use futures::StreamExt;
use gpui_kit::*;

use super::{KubeEvent, KubernetesModel};
use crate::settings;

impl KubernetesModel {
    /// Saves the settings. The next engine start, or Apply, uses them.
    pub fn save(&mut self, wanted: KubernetesSettings, cx: &mut Context<Self>) {
        self.engine.read(cx).set_kubernetes(wanted.clone());
        settings::update(cx, |settings| settings.kubernetes = wanted);
        cx.notify();
    }

    /// Turns Kubernetes on or off in the settings. Turning it on picks the stable
    /// version when none is saved, so the cluster never upgrades by itself.
    pub fn turn_on(&mut self, on: bool, cx: &mut Context<Self>) {
        let mut wanted = self.settings(cx);
        wanted.enabled = on;
        if on && wanted.version.is_none() {
            wanted.version = self.versions.stable().map(ToString::to_string);
        }
        self.save(wanted, cx);
    }

    /// Applies the saved settings to the running engine now: starts or restarts k3s,
    /// or stops it.
    pub fn apply(&mut self, cx: &mut Context<Self>) {
        if self.is_busy() || !self.engine_running(cx) {
            return;
        }
        let wanted = self.settings(cx);
        if wanted.enabled {
            let stream = self.host.enable(wanted);
            self.run("Apply", stream, cx);
        } else {
            let disable = self.host.disable();
            let stream = futures::stream::once(async move {
                disable.await.map(|()| "Kubernetes is off.".to_string())
            });
            self.run("Apply", stream.boxed(), cx);
        }
    }

    /// Deletes the cluster's workloads and state, then starts it again if it is on.
    pub fn reset(&mut self, cx: &mut Context<Self>) {
        if self.is_busy() || !self.engine_running(cx) {
            return;
        }
        let stream = self.host.reset();
        self.run("Reset", stream, cx);
    }

    /// Reads the version list; `refresh` asks the network.
    pub fn load_versions(&mut self, refresh: bool, cx: &mut Context<Self>) {
        let load = self.host.versions(refresh);
        cx.spawn(async move |this, cx| {
            if let Ok(versions) = load.await {
                this.update(cx, |model, cx| {
                    model.versions = versions;
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    /// Shows each progress line as the step, then reads the state again.
    fn run(
        &mut self,
        action: &'static str,
        mut stream: HostStream<String>,
        cx: &mut Context<Self>,
    ) {
        self.step = Some("Working…".into());
        cx.notify();
        self.task = Some(cx.spawn(async move |this, cx| {
            let mut failure = None;
            while let Some(line) = stream.next().await {
                match line {
                    Ok(line) => {
                        this.update(cx, |model, cx| {
                            model.step = Some(line.into());
                            cx.notify();
                        })
                        .ok();
                    }
                    Err(error) => {
                        failure = Some(error.0);
                        break;
                    }
                }
            }
            let status = this.read_with(cx, |model, _| model.host.status()).ok();
            let status = match status {
                Some(status) => status.await.ok(),
                None => None,
            };
            this.update(cx, |model, cx| {
                model.task = None;
                model.step = None;
                if let Some(status) = status {
                    model.set_status(status, cx);
                }
                if let Some(message) = failure {
                    cx.emit(KubeEvent { action, message });
                }
                cx.notify();
            })
            .ok();
        }));
    }
}

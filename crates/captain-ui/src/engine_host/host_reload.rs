//! Engine settings that changed in `settings.json` outside Captain. See ADR 0013.

use captain_core::HostResources;
use captain_core::settings::Settings;
use gpui_kit::*;

use super::{HostEvent, HostModel};

impl HostModel {
    /// Hands the engine settings that differ between `before` and `after` to
    /// Captain Engine for its next start, as the Settings page does. While the
    /// engine runs with other Docker daemon settings, the card shows "Restart to
    /// apply".
    pub fn follow_file(&mut self, before: &Settings, after: &Settings, cx: &mut Context<Self>) {
        if before.engine_daemon != after.engine_daemon {
            self.host.set_daemon(after.engine_daemon.clone());
        }
        if before.kubernetes != after.kubernetes {
            self.host.set_kubernetes(after.kubernetes.clone());
        }
        if before.engine_resources != after.engine_resources && !self.snapshotting {
            let resources = after.engine_resources.unwrap_or_else(|| {
                HostResources::recommended(self.machine.cpus, self.machine.memory_bytes)
            });
            let apply = self.host.set_resources(resources);
            cx.spawn(async move |this, cx| {
                if let Err(error) = apply.await {
                    this.update(cx, |_, cx| {
                        cx.emit(HostEvent::Failed {
                            action: "Resize",
                            message: error.0,
                        })
                    })
                    .ok();
                }
            })
            .detach();
        }
        cx.notify();
    }
}

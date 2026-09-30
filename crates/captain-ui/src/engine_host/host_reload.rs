//! Engine settings that changed in `settings.json` outside Captain. See ADR 0013.

use captain_core::settings::Settings;
use captain_core::{HostError, HostResources};
use gpui_kit::*;

use super::{HostEvent, HostModel};

impl HostModel {
    /// Hands the engine settings that differ between `before` and `after` to
    /// Captain Engine for its next start, as the Settings page does. While the
    /// engine runs with other Docker daemon settings, the card shows "Restart to
    /// apply". New resources wait while a snapshot step runs, and the end of the
    /// step hands them over before it starts the engine again.
    pub fn follow_file(&mut self, before: &Settings, after: &Settings, cx: &mut Context<Self>) {
        if before.engine_daemon != after.engine_daemon {
            self.host.set_daemon(after.engine_daemon.clone());
        }
        if before.kubernetes != after.kubernetes {
            self.host.set_kubernetes(after.kubernetes.clone());
        }
        if before.engine_resources != after.engine_resources {
            let resources = after.engine_resources.unwrap_or_else(|| {
                HostResources::recommended(self.machine.cpus, self.machine.memory_bytes)
            });
            match self.snapshotting {
                true => self.held_resources = Some(resources),
                false => {
                    let apply = self.host.set_resources(resources);
                    cx.spawn(async move |this, cx| {
                        let result = apply.await;
                        this.update(cx, |_, cx| report_resize(result, cx)).ok();
                    })
                    .detach();
                }
            }
        }
        cx.notify();
    }
}

/// Tells the user when the host could not take new resources.
pub(super) fn report_resize(result: Result<(), HostError>, cx: &mut Context<HostModel>) {
    if let Err(error) = result {
        cx.emit(HostEvent::Failed {
            action: "Resize",
            message: error.0,
        });
    }
}

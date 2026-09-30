use std::time::Duration;

use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::VolumesView;
use crate::widgets::danger_footer;

impl VolumesView {
    /// Removes unused volumes and shows what went as a notice. Without `all`, the
    /// engine removes only anonymous volumes.
    /// `generation` is the engine the button or confirmation was shown for.
    pub(super) fn prune(&mut self, all: bool, generation: u64, cx: &mut Context<Self>) {
        let Some(engine) = self
            .engine
            .clone()
            .filter(|_| generation == self.generation)
        else {
            return;
        };
        if self.pruning {
            return;
        }
        self.pruning = true;
        self.error = None;
        self.notice = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = engine.prune_unused_volumes(all, None).await;
            this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                this.pruning = false;
                match result {
                    Ok(report) => {
                        this.notice = Some(report.summary());
                        this.reload(Duration::ZERO, cx);
                    }
                    Err(error) => this.error = Some(format!("Prune failed: {error}")),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Asks before a prune that also removes named volumes, because their data is lost.
    pub(super) fn confirm_prune_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.entity().downgrade();
        let generation = self.generation;
        window.open_alert_dialog(cx, move |alert, _, _| {
            let view = view.clone();
            alert
                .title("Prune all unused volumes?")
                .description(
                    "This removes every volume that no container uses, named volumes \
                     included. Their data cannot be recovered.",
                )
                .footer(danger_footer(
                    "Prune all",
                    "Delete every volume that no container uses, and its data.",
                ))
                .on_ok(move |_, _, cx| {
                    view.update(cx, |view, cx| view.prune(true, generation, cx))
                        .ok();
                    true
                })
        });
    }
}

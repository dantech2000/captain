use std::time::Duration;

use gpui_kit::component::WindowExt;
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::*;

use super::VolumesView;

impl VolumesView {
    /// Removes unused volumes and shows what went as a notice. Without `all`, the
    /// engine removes only anonymous volumes.
    pub(super) fn prune(&mut self, all: bool, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
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
        window.open_alert_dialog(cx, move |alert, _, _| {
            let view = view.clone();
            alert
                .title("Prune all unused volumes?")
                .description(
                    "This removes every volume that no container uses, named volumes \
                     included. Their data cannot be recovered.",
                )
                .show_cancel(true)
                .ok_text("Prune all")
                .ok_variant(ButtonVariant::Danger)
                .on_ok(move |_, _, cx| {
                    view.update(cx, |view, cx| view.prune(true, cx)).ok();
                    true
                })
        });
    }
}

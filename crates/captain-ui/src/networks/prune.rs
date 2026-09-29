use std::time::Duration;

use captain_core::store::network_prune_summary;
use gpui_kit::*;

use super::NetworksView;

impl NetworksView {
    /// Removes custom networks without containers and shows how many went as a notice.
    /// Networks hold no data, so this does not ask first.
    pub(super) fn prune(&mut self, generation: u64, cx: &mut Context<Self>) {
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
            let result = engine.prune_unused_networks(None).await;
            this.update(cx, |this, cx| {
                this.pruning = false;
                match result {
                    Ok(removed) => {
                        this.notice = Some(network_prune_summary(&removed));
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
}

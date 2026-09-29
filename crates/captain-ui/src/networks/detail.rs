use captain_core::model::NetworkDetail;
use gpui_kit::*;

use super::NetworksView;

/// The inspect result of one network, or why it could not be loaded.
pub struct LoadedDetail {
    /// The network the result belongs to.
    pub id: String,
    pub result: Result<NetworkDetail, String>,
}

impl NetworksView {
    /// Inspects the selected network. A newer call cancels a pending one. The old
    /// result stays until the new one arrives, so the panel does not flicker on a reload.
    pub(super) fn load_detail(&mut self, cx: &mut Context<Self>) {
        let (Some(engine), Some(id)) = (self.engine.clone(), self.selected.clone()) else {
            self.detail = None;
            self.detail_task = None;
            return;
        };
        self.detail_task = Some(cx.spawn(async move |this, cx| {
            let result = engine.inspect_network(&id).await;
            this.update(cx, |this, cx| {
                if this.selected.as_deref() != Some(id.as_str()) {
                    return;
                }
                let result = result.map_err(|error| {
                    tracing::warn!(%error, network = %id, "could not inspect network");
                    error.to_string()
                });
                this.detail = Some(LoadedDetail { id, result });
                cx.notify();
            })
            .ok();
        }));
    }

    /// The loaded detail of the network `id`, if it belongs to it.
    pub(super) fn detail_of(&self, id: &str) -> Option<&Result<NetworkDetail, String>> {
        self.detail
            .as_ref()
            .filter(|detail| detail.id == id)
            .map(|detail| &detail.result)
    }
}

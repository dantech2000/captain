use captain_core::model::{ImageDetail, ImageLayer};
use gpui_kit::*;

use super::ImagesState;

impl ImagesState {
    /// Loads the details and history of image `id`. A newer call cancels a pending one.
    pub(super) fn load_detail(&mut self, id: &str, cx: &mut Context<Self>) {
        self.clear_detail();
        let Some(engine) = self.engine.clone() else {
            return;
        };
        let inspect = engine.inspect_image(id);
        let history = engine.image_history(id);
        self.detail_task = Some(cx.spawn(async move |this, cx| {
            let (detail, layers) = futures::join!(inspect, history);
            this.update(cx, |this, cx| {
                match (detail, layers) {
                    (Ok(detail), Ok(layers)) => {
                        this.detail = Some(detail);
                        this.layers = Some(layers);
                    }
                    (Err(error), _) | (_, Err(error)) => {
                        tracing::warn!(%error, "inspecting an image failed");
                        this.detail_error = Some(format!("Could not load details: {error}"));
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    pub(super) fn clear_detail(&mut self) {
        self.detail = None;
        self.layers = None;
        self.detail_error = None;
        self.detail_task = None;
    }

    /// The details of the selected image, once they have loaded.
    pub fn detail(&self) -> Option<&ImageDetail> {
        self.detail.as_ref()
    }

    /// The history of the selected image, once it has loaded.
    pub fn layers(&self) -> Option<&[ImageLayer]> {
        self.layers.as_deref()
    }

    pub fn detail_error(&self) -> Option<&str> {
        self.detail_error.as_deref()
    }
}

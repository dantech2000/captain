use std::time::Duration;

use captain_core::EngineError;
use captain_core::model::{EngineEvent, Image};
use captain_core::store::changes_image_list;
use gpui_kit::*;

use super::ImagesState;

/// Events often come in bursts, for example a pull tags and untags images. Wait this
/// long after the last event before reloading the list.
const RELOAD_DEBOUNCE: Duration = Duration::from_millis(150);

impl ImagesState {
    /// Reloads the image list after `delay`. A newer call cancels a pending one.
    pub(super) fn reload(&mut self, delay: Duration, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            if !delay.is_zero() {
                cx.background_executor().timer(delay).await;
            }
            let result = engine.list_images().await;
            this.update(cx, |this, cx| this.apply(result, cx)).ok();
        }));
    }

    fn apply(&mut self, result: Result<Vec<Image>, EngineError>, cx: &mut Context<Self>) {
        match result {
            Ok(images) => {
                self.store.replace(images);
                self.loaded = true;
                self.load_error = None;
                let valid = self
                    .selected
                    .as_deref()
                    .is_some_and(|id| self.store.find(id).is_some());
                if !valid {
                    self.selected = None;
                    self.clear_detail();
                }
            }
            Err(error) => {
                tracing::warn!(%error, "listing images failed");
                self.load_error = Some(format!("Could not list images: {error}"));
            }
        }
        cx.notify();
    }

    /// Reloads the list when an engine event changes it.
    pub fn on_engine_event(&mut self, event: &EngineEvent, cx: &mut Context<Self>) {
        if changes_image_list(event) {
            self.reload(RELOAD_DEBOUNCE, cx);
        }
    }
}

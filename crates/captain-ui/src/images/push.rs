use std::sync::Arc;
use std::time::Duration;

use captain_core::ImageBuilder;
use captain_core::store::PushTracker;
use futures::StreamExt;
use gpui_kit::*;

use super::ImagesState;

impl ImagesState {
    /// Pushes the tag `reference` with the login from the Docker config. Only one push
    /// runs at a time.
    pub fn push_image(&mut self, reference: String, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        if self.is_pushing() {
            return;
        }
        let mut messages = engine.push_image(&reference);
        self.push = Some(PushTracker::new(reference));
        self.push_error = None;
        cx.notify();
        self.push_task = Some(cx.spawn(async move |this, cx| {
            let mut failed = false;
            while let Some(message) = messages.next().await {
                let updated = this.update(cx, |this, cx| {
                    match message {
                        Ok(message) => {
                            if let Some(push) = this.push.as_mut() {
                                push.apply(&message);
                            }
                        }
                        Err(error) => {
                            tracing::warn!(%error, "pushing an image failed");
                            this.push_error = Some(format!("Push failed: {error}"));
                            failed = true;
                        }
                    }
                    cx.notify();
                });
                if updated.is_err() || failed {
                    break;
                }
            }
            this.update(cx, |this, cx| {
                if !failed && let Some(push) = this.push.as_mut() {
                    push.finish();
                }
                this.push_task = None;
                cx.notify();
            })
            .ok();
        }));
    }

    /// The running or last finished push.
    pub fn push(&self) -> Option<&PushTracker> {
        self.push.as_ref()
    }

    /// True while a push stream is open.
    pub fn is_pushing(&self) -> bool {
        self.push_task.is_some()
    }

    pub fn push_error(&self) -> Option<&str> {
        self.push_error.as_deref()
    }

    /// The image builder, if `docker buildx` is installed.
    pub fn builder(&self) -> Option<Arc<dyn ImageBuilder>> {
        self.builder.clone()
    }

    /// Shows a result in green, for example "Tagged app:1.0", and reloads the list.
    /// Does nothing if the engine changed since `generation`.
    pub fn set_notice(&mut self, notice: String, generation: u64, cx: &mut Context<Self>) {
        if generation != self.generation {
            return;
        }
        self.error = None;
        self.notice = Some(notice);
        self.reload(Duration::ZERO, cx);
        cx.notify();
    }
}

use std::time::Duration;

use captain_core::format::bytes_label;
use captain_core::model::ImageReference;
use captain_core::store::PullTracker;
use futures::StreamExt;
use gpui_kit::*;

use super::ImagesState;

impl ImagesState {
    /// Removes one image. It never forces, so the engine refuses an image in use.
    pub fn remove(&mut self, id: String, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        if !self.removing.insert(id.clone()) {
            return;
        }
        self.error = None;
        self.notice = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = engine.remove_image(&id).await;
            this.update(cx, |this, cx| {
                this.removing.remove(&id);
                if let Err(error) = result {
                    tracing::warn!(%error, "removing an image failed");
                    this.error = Some(format!("Remove failed: {error}"));
                }
                this.reload(Duration::ZERO, cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Removes every dangling image, then shows how much space that freed.
    pub fn prune_dangling(&mut self, cx: &mut Context<Self>) {
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
            let result = engine.prune_dangling_images().await;
            this.update(cx, |this, cx| {
                this.pruning = false;
                match result {
                    Ok(bytes) => {
                        this.notice = Some(format!("Reclaimed {}", bytes_label(bytes)));
                    }
                    Err(error) => {
                        tracing::warn!(%error, "pruning images failed");
                        this.error = Some(format!("Prune failed: {error}"));
                    }
                }
                this.reload(Duration::ZERO, cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Pulls `input`, for example `nginx:alpine`. Only one pull runs at a time.
    pub fn pull_image(&mut self, input: &str, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        if self.is_pulling() {
            return;
        }
        let Some(reference) = ImageReference::parse(input) else {
            self.pull = None;
            self.pull_error = Some("Enter an image name, for example nginx:alpine.".into());
            cx.notify();
            return;
        };
        let reference = reference.to_string();
        let mut messages = engine.pull_image(&reference);
        self.pull = Some(PullTracker::new(reference));
        self.pull_error = None;
        cx.notify();
        self.pull_task = Some(cx.spawn(async move |this, cx| {
            let mut failed = false;
            while let Some(message) = messages.next().await {
                let updated = this.update(cx, |this, cx| {
                    match message {
                        Ok(message) => {
                            if let Some(pull) = this.pull.as_mut() {
                                pull.apply(&message);
                            }
                        }
                        Err(error) => {
                            tracing::warn!(%error, "pulling an image failed");
                            this.pull_error = Some(format!("Pull failed: {error}"));
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
                if !failed && let Some(pull) = this.pull.as_mut() {
                    pull.finish();
                }
                this.pull_task = None;
                this.reload(Duration::ZERO, cx);
                cx.notify();
            })
            .ok();
        }));
    }
}

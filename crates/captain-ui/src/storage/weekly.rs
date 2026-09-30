//! The weekly build-cache cleanup. It runs while the engine is connected, when the
//! setting is on and a week has passed since the last run.

use std::time::Duration;

use captain_core::format::bytes_label;
use captain_core::storage::BUILD_CACHE_AGE;
use gpui_kit::*;

use super::StorageModel;
use super::storage_model::now;
use crate::settings;

/// How often Captain checks whether a run is due.
const CHECK: Duration = Duration::from_secs(60 * 60);
/// The first check waits for the engine to connect and settle.
const FIRST_CHECK: Duration = Duration::from_secs(2 * 60);
const WEEK: i64 = 7 * 24 * 60 * 60;

impl StorageModel {
    pub(super) fn start_weekly(&mut self, cx: &mut Context<Self>) {
        self._weekly = Some(cx.spawn(async move |this, cx| {
            let mut delay = FIRST_CHECK;
            loop {
                cx.background_executor().timer(delay).await;
                delay = CHECK;
                if this.update(cx, |model, cx| model.run_weekly(cx)).is_err() {
                    return;
                }
            }
        }));
    }

    fn run_weekly(&mut self, cx: &mut Context<Self>) {
        let saved = settings::current(cx);
        let now = now();
        let due = saved
            .build_cache_cleaned_at
            .is_none_or(|last| now - last >= WEEK);
        let Some(engine) = self.engine.clone() else {
            return;
        };
        if !saved.weekly_build_cache_cleanup || !due || self.step.is_some() {
            return;
        }
        // Saved first, so a failing engine is not asked again every hour.
        settings::update(cx, |settings| settings.build_cache_cleaned_at = Some(now));
        cx.spawn(async move |this, cx| {
            match engine.prune_build_cache(BUILD_CACHE_AGE).await {
                Ok(bytes) => {
                    tracing::info!(freed = %bytes_label(bytes), "weekly build cache cleanup")
                }
                Err(error) => tracing::warn!(%error, "weekly build cache cleanup failed"),
            }
            this.update(cx, |model, cx| model.reload(cx)).ok();
        })
        .detach();
    }
}

use std::sync::Arc;
use std::time::Duration;

use captain_core::model::Network;
use captain_core::store::NetworkStore;
use captain_core::{Engine, EngineError};
use gpui_kit::*;

use crate::workspace::Page;

use super::NetworksView;

/// Events often come in bursts, for example `docker compose up`. Wait this long
/// after the last event before reloading the list.
pub(super) const RELOAD_DEBOUNCE: Duration = Duration::from_millis(250);

impl NetworksView {
    /// Starts loading and following events when the workspace connects to an engine,
    /// and stops when it loses it.
    pub(super) fn follow_engine(
        &mut self,
        engine: Option<Arc<dyn Engine>>,
        cx: &mut Context<Self>,
    ) {
        let same = match (&self.engine, &engine) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        if same {
            return;
        }
        self.engine = engine;
        self.reload_task = None;
        self.clear_engine_state();
        if self.engine.is_some() {
            self.reload(Duration::ZERO, cx);
        }
    }

    /// Forgets everything loaded from the old engine at once, so no row or selection
    /// of it can act on the new one.
    fn clear_engine_state(&mut self) {
        self.generation += 1;
        self.store = NetworkStore::default();
        self.loaded = false;
        self.selected = None;
        self.removing.clear();
        self.detail = None;
        self.detail_task = None;
        self.error = None;
        self.notice = None;
    }

    /// Reloads the network list after `delay`. A newer call cancels a pending one.
    pub(super) fn reload(&mut self, delay: Duration, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            if !delay.is_zero() {
                cx.background_executor().timer(delay).await;
            }
            let result = engine.list_networks().await;
            this.update(cx, |this, cx| this.apply(result, cx)).ok();
        }));
    }

    fn apply(&mut self, result: Result<Vec<Network>, EngineError>, cx: &mut Context<Self>) {
        match result {
            Ok(networks) => {
                self.store.replace(networks);
                let count = self.store.len();
                self.workspace.update(cx, |workspace, cx| {
                    workspace.set_page_count(Page::Networks, count, cx)
                });
                self.loaded = true;
                if self.error.as_deref().is_some_and(is_load_error) {
                    self.error = None;
                }
                let valid = self
                    .selected
                    .as_deref()
                    .is_some_and(|id| self.store.find(id).is_some());
                if !valid {
                    self.selected = None;
                }
                // Connects and disconnects reload the list, and change the endpoints.
                self.load_detail(cx);
            }
            Err(error) => {
                tracing::warn!(%error, "could not list networks");
                self.error = Some(format!("{LOAD_ERROR}{error}"));
            }
        }
        cx.notify();
    }
}

const LOAD_ERROR: &str = "Could not load networks: ";

fn is_load_error(message: &str) -> bool {
    message.starts_with(LOAD_ERROR)
}

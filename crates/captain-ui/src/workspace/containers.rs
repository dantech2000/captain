use std::time::{Duration, Instant};

use captain_core::EngineError;
use captain_core::model::Container;
use captain_core::store::ContainerFilter;
use futures::StreamExt;
use gpui_kit::*;

use super::Workspace;

/// Events often come in bursts, for example `docker compose up`. Wait this long after
/// the last event before reloading the list.
const RELOAD_DEBOUNCE: Duration = Duration::from_millis(150);
/// The engine's status text ("Up 3 minutes") only changes when Captain lists again,
/// and a running container sends no events. Reload this often while any runs.
const STATUS_REFRESH: Duration = Duration::from_secs(30);

impl Workspace {
    /// Reloads the container list after `delay`. A newer call cancels a pending one.
    pub(super) fn reload(&mut self, delay: Duration, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            if !delay.is_zero() {
                cx.background_executor().timer(delay).await;
            }
            let result = engine.list_containers().await;
            this.update(cx, |this, cx| this.apply(result, cx)).ok();
        }));
    }

    fn apply(&mut self, result: Result<Vec<Container>, EngineError>, cx: &mut Context<Self>) {
        match result {
            Ok(containers) => {
                self.store.replace(containers);
                self.loaded = true;
                self.keep_selection_valid();
                let store = &self.store;
                self.checked.retain(|id| store.find(id).is_some());
                self.sync_stats(cx);
                if self.store.active_count() > 0 {
                    self.reload(STATUS_REFRESH, cx);
                }
                cx.notify();
            }
            Err(error) => self.fail(error, cx),
        }
    }

    /// Selects the first container in list order if nothing valid is selected.
    pub(super) fn keep_selection_valid(&mut self) {
        let valid = self
            .selected
            .as_deref()
            .is_some_and(|id| self.store.find(id).is_some());
        if !valid {
            self.selected = self
                .store
                .groups(ContainerFilter::All, self.show_kubernetes)
                .first()
                .and_then(|g| g.containers.first())
                .map(|c| c.id.clone());
        }
    }

    /// Notifies once when the next recent crash expires, then waits for the one
    /// after it. A crash counts for a minute only, and with no container running
    /// nothing else redraws the tray, Dock badge, and sidebar.
    fn schedule_crash_expiry(&mut self, cx: &mut Context<Self>) {
        let Some(wait) = self.crashes.next_expiry(Instant::now()) else {
            self.crash_expiry = None;
            return;
        };
        self.crash_expiry = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(wait).await;
            this.update(cx, |this, cx| {
                cx.notify();
                this.schedule_crash_expiry(cx);
            })
            .ok();
        }));
    }

    pub(super) fn watch_events(&mut self, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        let mut events = engine.events();
        self.events_task = Some(cx.spawn(async move |this, cx| {
            while let Some(event) = events.next().await {
                let updated = match event {
                    Ok(event) => this.update(cx, |this, cx| {
                        this.crashes.record(&event);
                        if event.action == "die" {
                            this.schedule_crash_expiry(cx);
                        }
                        if event.changes_container_list() {
                            this.reload(RELOAD_DEBOUNCE, cx);
                        }
                        cx.emit(event);
                    }),
                    Err(error) => this.update(cx, |this, cx| this.fail(error, cx)),
                };
                if updated.is_err() {
                    return;
                }
            }
            let closed = EngineError::Unreachable("the engine closed the event stream".into());
            this.update(cx, |this, cx| this.fail(closed, cx)).ok();
        }));
    }
}

//! When and how often Captain reconnects by itself after the engine stops
//! answering, for example after `systemctl restart docker` in the engine. See
//! feature 0013.

use std::time::{Duration, Instant};

use captain_core::store::SelectMode;
use captain_core::{EngineError, HostStatus};
use gpui_kit::*;

use super::{Connection, Connector, Workspace};
use crate::engine_host;

/// The wait before the first automatic attempt. Each failed attempt doubles it.
const FIRST_WAIT: Duration = Duration::from_secs(1);
/// The longest wait between two attempts.
const MAX_WAIT: Duration = Duration::from_secs(30);
/// After a working connection drops, pages show "Reconnecting" this long before
/// they show the failure.
pub(super) const QUIET: Duration = Duration::from_secs(15);
/// After a connect, the event stream counts as working once it delivers an event or
/// stays open this long. An engine or proxy that refuses `/events` fails it at once.
const SETTLE: Duration = Duration::from_secs(5);

/// The wait before automatic attempt `attempt`, counted from 0: 1 s, 2 s, 4 s, and
/// so on, up to 30 s.
pub fn backoff(attempt: u32) -> Duration {
    FIRST_WAIT
        .checked_mul(2u32.saturating_pow(attempt))
        .map_or(MAX_WAIT, |wait| wait.min(MAX_WAIT))
}

/// True if Captain should reconnect by itself: the settings choose another engine
/// (`host` is `None`), or Captain Engine runs. While Captain Engine is stopped or
/// starts, the host model connects when it runs.
pub fn should_auto_reconnect(host: Option<&HostStatus>) -> bool {
    host.is_none_or(HostStatus::is_running)
}

/// The endpoint for an automatic reconnect: Captain Engine's endpoint when the
/// settings choose it, else the endpoint of the last working connection, else the
/// saved one (`None` means discovery). Only an explicit reconnect discovers again,
/// so a changed Docker context cannot switch engines under the focused project.
pub fn resume_endpoint(
    captain: Option<String>,
    connected: Option<String>,
    saved: Option<String>,
) -> Option<String> {
    captain.or(connected).or(saved)
}

/// The automatic reconnect in progress, until the containers load and the event
/// stream works, or the user reconnects or switches engines.
pub(super) struct AutoReconnect {
    /// The number of failed attempts so far.
    pub attempt: u32,
    /// When the working connection dropped, or `None` if the engine never answered
    /// since the last manual connect. Only a drop gets the quiet period.
    pub dropped_at: Option<Instant>,
    /// The last error.
    pub error: EngineError,
    /// Waits, then reconnects. `None` while the attempt runs.
    pub task: Option<Task<()>>,
    /// Shows the failure when the quiet period ends, also while an attempt runs.
    pub quiet_end: Option<Task<()>>,
    /// True once the event stream of the current connection works. See [`SETTLE`].
    pub events_working: bool,
    /// Marks the event stream as working after [`SETTLE`].
    pub settle: Option<Task<()>>,
}

impl AutoReconnect {
    /// The time left in the quiet period, or `None` when there is none or it ended.
    pub fn quiet_left(&self, now: Instant) -> Option<Duration> {
        let end = self.dropped_at? + QUIET;
        end.checked_duration_since(now)
            .filter(|left| !left.is_zero())
    }

    /// True while pages should still show "Reconnecting" instead of the failure.
    pub fn is_quiet(&self, now: Instant) -> bool {
        self.quiet_left(now).is_some()
    }
}

/// True if the engine the settings choose should answer now. See
/// [`should_auto_reconnect`]. It reads the host model, so it must not run inside
/// the host model's update.
pub fn expected_running(cx: &App) -> bool {
    let status = engine_host::summary(cx).map(|host| host.status);
    should_auto_reconnect(status.as_ref())
}

impl Workspace {
    /// The endpoint of the last working connection, kept while Captain reconnects.
    pub fn connected_endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }

    /// The last error while Captain reconnects by itself and has no connection,
    /// else `None`.
    pub fn reconnecting(&self) -> Option<&EngineError> {
        if matches!(self.connection, Connection::Connected(_)) {
            return None;
        }
        self.auto.as_ref().map(|auto| &auto.error)
    }

    /// Drops the engine's tasks after `error`. When the engine should answer, it
    /// tries again after a backoff, and a connection that worked shows
    /// "Reconnecting" for [`QUIET`] before the failure. A failure right after a
    /// reconnect, before the feeds worked, continues the backoff and the quiet
    /// period.
    pub(super) fn fail(&mut self, error: EngineError, cx: &mut Context<Self>) {
        self.events_task = None;
        self.reload_task = None;
        self.stats_tasks.clear();
        let dropped = matches!(self.connection, Connection::Connected(_));
        if !expected_running(cx) {
            tracing::warn!(%error, "engine connection failed");
            self.auto = None;
            self.connection = Connection::Failed(error);
            cx.notify();
            return;
        }
        let now = Instant::now();
        let auto = self.auto.get_or_insert_with(|| AutoReconnect {
            attempt: 0,
            dropped_at: dropped.then_some(now),
            error: error.clone(),
            task: None,
            quiet_end: None,
            events_working: false,
            settle: None,
        });
        auto.events_working = false;
        auto.settle = None;
        if auto.task.is_some() {
            // The engine's reload and event stream can both fail; one wait is enough.
            auto.error = error;
            return;
        }
        if auto.quiet_end.is_none()
            && let Some(left) = auto.quiet_left(now)
        {
            auto.quiet_end = Some(cx.spawn(async move |this, cx| {
                cx.background_executor().timer(left).await;
                this.update(cx, |this, cx| this.end_quiet(cx)).ok();
            }));
        }
        let wait = backoff(auto.attempt);
        if auto.attempt == 0 {
            tracing::warn!(%error, ?wait, "engine connection failed; reconnecting");
        } else {
            tracing::debug!(%error, ?wait, attempt = auto.attempt, "reconnect failed");
        }
        auto.attempt = auto.attempt.saturating_add(1);
        auto.error = error.clone();
        self.connection = if auto.is_quiet(now) {
            Connection::Connecting
        } else {
            Connection::Failed(error)
        };
        auto.task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(wait).await;
            cx.update(|cx| {
                if let Some(workspace) = this.upgrade() {
                    crate::settings::resume(&workspace, cx);
                }
            });
        }));
        cx.notify();
    }

    /// Connects again with `connect` after a drop. Unlike
    /// [`Workspace::reconnect`], it keeps the selected container and the backoff,
    /// and the pages keep "Reconnecting" or the failure until the engine answers.
    pub fn resume(&mut self, connect: Connector, cx: &mut Context<Self>) {
        let selected = self.selected.take();
        let connection = self.connection.clone();
        let mut auto = self.auto.take();
        if let Some(auto) = auto.as_mut() {
            auto.task = None;
        }
        self.drop_engine();
        if let Some(id) = &selected {
            self.checked.click(id, SelectMode::Replace, &[]);
        }
        self.selected = selected;
        self.connection = connection;
        self.auto = auto;
        cx.notify();
        self.connect(connect, cx);
    }

    /// Shows the failure when the quiet period ends while Captain still has no
    /// connection, even while an attempt runs. Retries go on behind it.
    fn end_quiet(&mut self, cx: &mut Context<Self>) {
        if let Some(auto) = &self.auto
            && matches!(self.connection, Connection::Connecting)
        {
            self.connection = Connection::Failed(auto.error.clone());
            cx.notify();
        }
    }

    /// After a connect during an automatic reconnect, waits until the event stream
    /// works. The backoff stays until then. See [`Workspace::finish_reconnect`].
    pub(super) fn watch_feeds(&mut self, cx: &mut Context<Self>) {
        let Some(auto) = self.auto.as_mut() else {
            return;
        };
        auto.events_working = false;
        auto.settle = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SETTLE).await;
            this.update(cx, |this, _| this.events_work()).ok();
        }));
    }

    /// The event stream delivered an event or stayed open for [`SETTLE`].
    pub(super) fn events_work(&mut self) {
        let Some(auto) = self.auto.as_mut() else {
            return;
        };
        auto.events_working = true;
        auto.settle = None;
        self.finish_reconnect();
    }

    /// Ends the automatic reconnect, and with it the backoff, once the containers
    /// loaded and the event stream works.
    pub(super) fn finish_reconnect(&mut self) {
        let working = self.loaded
            && matches!(self.connection, Connection::Connected(_))
            && self.auto.as_ref().is_some_and(|auto| auto.events_working);
        if working {
            self.auto = None;
            tracing::info!("reconnected to the engine");
        }
    }

    /// Stops reconnecting by itself, for example because Captain Engine stopped.
    /// Pages that showed "Reconnecting" show the failure.
    pub fn stop_reconnecting(&mut self, cx: &mut Context<Self>) {
        if let Some(auto) = self.auto.take()
            && matches!(self.connection, Connection::Connecting)
        {
            self.connection = Connection::Failed(auto.error);
        }
        cx.notify();
    }
}

#[cfg(test)]
mod tests;

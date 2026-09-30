use std::time::{Duration, SystemTime, UNIX_EPOCH};

use captain_core::model::{LogLevel, LogOptions};
use captain_core::store::LevelFilter;
use gpui_kit::*;

use super::LogsPane;
use super::logs_pane::LOG_TAIL;

impl LogsPane {
    /// Loads the lines again from `since` ago on, and shows only errors if `errors`,
    /// else every level. A `logs` command in the ⌘K palette sets them.
    pub fn filter(&mut self, since: Option<Duration>, errors: bool, cx: &mut Context<Self>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        self.since = since.map(|since| (since, now.saturating_sub(since).as_secs() as i64));
        self.level = if errors {
            LevelFilter::Only(LogLevel::Error)
        } else {
            LevelFilter::All
        };
        self.reconnect(cx);
    }

    /// The time filter a command set, if any.
    pub(super) fn since(&self) -> Option<Duration> {
        self.since.map(|(since, _)| since)
    }

    /// Keeps the time filter while the same container restarts or reloads, and
    /// drops it for another container.
    pub(super) fn keep_since_for(&mut self, id: &str) {
        if self.source.as_ref().is_none_or(|(_, old)| old != id) {
            self.since = None;
        }
    }

    /// Drops the time filter and loads the recent lines again.
    pub(super) fn clear_since(&mut self, cx: &mut Context<Self>) {
        self.since = None;
        self.reconnect(cx);
    }

    /// The recent lines, or every line since the time filter.
    pub(super) fn log_options(&self) -> LogOptions {
        let since = self.since.map(|(_, time)| time);
        LogOptions {
            tail: since.is_none().then_some(LOG_TAIL),
            since,
            follow: true,
        }
    }
}

#[cfg(test)]
mod tests;

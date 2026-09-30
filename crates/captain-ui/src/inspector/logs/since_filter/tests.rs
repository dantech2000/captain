use std::sync::Arc;
use std::time::Duration;

use captain_core::FakeEngine;

use super::super::LogsPane;

#[test]
fn the_time_filter_stays_for_the_same_container_and_clears_for_another() {
    let mut pane = LogsPane::default();
    pane.source = Some((Arc::new(FakeEngine::default()), "worker".into()));
    pane.since = Some((Duration::from_secs(600), 1_790_000_000));
    pane.keep_since_for("worker");
    assert!(pane.since.is_some());
    pane.keep_since_for("api");
    assert!(pane.since.is_none());
}

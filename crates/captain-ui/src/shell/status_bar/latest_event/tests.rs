use chrono::{Local, TimeZone};

use super::LatestEvent;

#[test]
fn counts_restarts_after_crashes_but_not_after_a_stop() {
    let at = Local.with_ymd_and_hms(2026, 9, 29, 12, 7, 11).unwrap();
    let mut events = LatestEvent::default();
    for action in ["kill", "die", "stop", "start"] {
        events.record(action, "web-id", "web", at);
    }
    assert_eq!(events.line(), None);

    for _ in 0..3 {
        events.record("die", "worker-id", "worker", at);
        events.record("start", "worker-id", "worker", at);
    }
    assert_eq!(
        events.line().as_deref(),
        Some("worker restarted 3 times · last at 12:07:11")
    );
}

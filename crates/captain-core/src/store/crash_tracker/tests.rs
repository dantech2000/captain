use std::time::{Duration, Instant};

use super::{CrashTracker, RECENT};
use crate::model::{EngineEvent, EventKind};

fn event(action: &str) -> EngineEvent {
    EngineEvent {
        kind: EventKind::Container,
        action: action.into(),
        id: "c1".into(),
        ..EngineEvent::default()
    }
}

#[test]
fn an_oom_exit_counts_as_a_recent_out_of_memory_crash() {
    let start = Instant::now();
    let mut tracker = CrashTracker::default();
    tracker.record_at(&event("oom"), start);
    tracker.record_at(&event("die"), start);
    tracker.record_at(&event("start"), start);
    let crash = tracker
        .recent("c1", start + Duration::from_secs(5))
        .unwrap();
    assert!(crash.out_of_memory);
    assert_eq!(tracker.recent("c1", start + RECENT), None);
}

#[test]
fn a_stop_or_kill_is_not_a_crash() {
    let at = Instant::now();
    let mut tracker = CrashTracker::default();
    tracker.record_at(&event("kill"), at);
    tracker.record_at(&event("die"), at);
    assert_eq!(tracker.recent("c1", at), None);
}

#[test]
fn stopping_a_crashed_container_clears_its_crash() {
    let at = Instant::now();
    let mut tracker = CrashTracker::default();
    tracker.record_at(&event("die"), at);
    assert!(tracker.recent("c1", at).is_some());
    tracker.record_at(&event("kill"), at);
    tracker.record_at(&event("die"), at);
    assert_eq!(tracker.recent("c1", at), None);
}

#[test]
fn next_expiry_is_when_the_oldest_recent_crash_stops_counting() {
    let start = Instant::now();
    let mut tracker = CrashTracker::default();
    assert_eq!(tracker.next_expiry(start), None);
    tracker.record_at(&event("die"), start);
    let later = start + Duration::from_secs(20);
    assert_eq!(
        tracker.next_expiry(later),
        Some(RECENT - Duration::from_secs(20))
    );
    assert_eq!(tracker.next_expiry(start + RECENT), None);
}

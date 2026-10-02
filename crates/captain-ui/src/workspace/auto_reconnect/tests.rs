use std::time::{Duration, Instant};

use captain_core::{EngineError, HostStatus};

use super::{AutoReconnect, QUIET, backoff, resume_endpoint, should_auto_reconnect};

#[test]
fn backoff_doubles_from_one_second_up_to_thirty() {
    let waits: Vec<u64> = (0..7).map(|attempt| backoff(attempt).as_secs()).collect();
    assert_eq!(waits, [1, 2, 4, 8, 16, 30, 30]);
    assert_eq!(backoff(u32::MAX), Duration::from_secs(30));
}

#[test]
fn reconnects_only_to_a_running_captain_engine_or_another_engine() {
    assert!(should_auto_reconnect(None));
    assert!(should_auto_reconnect(Some(&HostStatus::Running)));
    for status in [
        HostStatus::Stopped,
        HostStatus::Starting,
        HostStatus::Stopping,
        HostStatus::NotCreated,
        HostStatus::Failed("broken".into()),
    ] {
        assert!(!should_auto_reconnect(Some(&status)), "{status:?}");
    }
}

#[test]
fn an_automatic_reconnect_keeps_the_connected_endpoint() {
    let unix = || Some("unix:///var/run/docker.sock".to_string());
    assert_eq!(resume_endpoint(None, unix(), None), unix());
    let captain = Some("unix:///captain.sock".to_string());
    assert_eq!(resume_endpoint(captain.clone(), unix(), None), captain);
}

#[test]
fn the_quiet_period_ends_fifteen_seconds_after_the_drop() {
    let dropped = Instant::now();
    let auto = AutoReconnect {
        attempt: 3,
        dropped_at: Some(dropped),
        error: EngineError::Unreachable("gone".into()),
        task: None,
        quiet_end: None,
        events_working: false,
        settle: None,
    };
    let left = auto.quiet_left(dropped + Duration::from_secs(5));
    assert_eq!(left, Some(QUIET - Duration::from_secs(5)));
    assert_eq!(auto.quiet_left(dropped + QUIET), None);
}

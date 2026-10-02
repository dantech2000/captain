use std::time::Duration;

use captain_core::HostStatus;

use super::{backoff, should_auto_reconnect};

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

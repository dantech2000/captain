use std::time::Duration;

use super::InFlight;

#[test]
fn counts_until_the_guard_drops() {
    let in_flight = InFlight::default();
    let guard = in_flight.enter();
    let second = in_flight.clone().enter();
    assert_eq!(in_flight.count(), 2);
    drop(guard);
    drop(second);
    assert_eq!(in_flight.count(), 0);
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("runtime")
}

#[test]
fn wait_idle_returns_after_the_limit() {
    let in_flight = InFlight::default();
    let _guard = in_flight.enter();
    let idle = runtime().block_on(in_flight.wait_idle(Some(Duration::from_millis(250))));
    assert!(!idle);
    assert_eq!(in_flight.count(), 1);
}

#[test]
fn wait_idle_without_a_limit_waits_for_the_last_guard() {
    let in_flight = InFlight::default();
    let guard = in_flight.enter();
    let ended = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        drop(guard);
    });
    assert!(runtime().block_on(in_flight.wait_idle(None)));
    assert_eq!(in_flight.count(), 0);
    ended.join().expect("thread");
}

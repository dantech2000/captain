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

#[test]
fn wait_idle_returns_after_the_limit() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("runtime");
    let in_flight = InFlight::default();
    let _guard = in_flight.enter();
    runtime.block_on(in_flight.wait_idle(Duration::from_millis(250)));
    assert_eq!(in_flight.count(), 1);
}

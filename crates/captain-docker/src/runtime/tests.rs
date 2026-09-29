use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::channel::mpsc::UnboundedSender;

use super::{BackgroundRuntime, build, forward, spawn_to_end};

#[test]
fn dropping_a_stream_aborts_an_idle_producer() {
    let runtime = build().unwrap();
    let alive = Arc::new(());
    let held = alive.clone();
    // The producer sends nothing, so only an abort can stop it.
    let stream = forward(
        runtime.handle(),
        move |_tx: UnboundedSender<()>| async move {
            let _held = held;
            std::future::pending::<()>().await;
        },
    );
    drop(stream);
    let deadline = Instant::now() + Duration::from_secs(5);
    while Arc::strong_count(&alive) > 1 {
        assert!(Instant::now() < deadline, "the producer still runs");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn dropping_a_background_runtime_does_not_wait_for_blocking_work() {
    let runtime = BackgroundRuntime::from(build().unwrap());
    runtime.spawn_blocking(|| std::thread::sleep(Duration::from_secs(5)));
    let dropped = Instant::now();
    drop(runtime);
    assert!(dropped.elapsed() < Duration::from_secs(1));
}

#[test]
fn work_spawned_to_end_finishes_after_its_runtime_owner_drops() {
    let runtime = Arc::new(BackgroundRuntime::from(build().unwrap()));
    let work = spawn_to_end(runtime.clone(), async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(7)
    });
    drop(runtime);
    assert_eq!(futures::executor::block_on(work).unwrap(), 7);
}

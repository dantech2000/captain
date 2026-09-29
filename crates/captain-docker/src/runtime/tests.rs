use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::channel::mpsc::UnboundedSender;

use super::{build, forward};

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

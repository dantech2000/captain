use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::OutputBudget;

#[test]
fn a_full_queue_holds_the_reader_until_the_view_takes_output() {
    let budget = OutputBudget::new(10);
    let release = budget.release_handle();
    // An empty queue takes a chunk larger than the limit.
    assert!(budget.acquire(25));

    let (done_tx, done_rx) = mpsc::channel();
    let reader = budget.clone();
    thread::spawn(move || done_tx.send(reader.acquire(1)).ok());
    assert!(done_rx.recv_timeout(Duration::from_millis(50)).is_err());
    release.release(25);
    assert_eq!(done_rx.recv_timeout(Duration::from_secs(2)), Ok(true));

    // A view that stops reading lets a waiting reader go.
    assert!(budget.acquire(9));
    let (done_tx, done_rx) = mpsc::channel();
    let reader = budget.clone();
    thread::spawn(move || done_tx.send(reader.acquire(5)).ok());
    drop(release);
    assert_eq!(done_rx.recv_timeout(Duration::from_secs(2)), Ok(false));
}

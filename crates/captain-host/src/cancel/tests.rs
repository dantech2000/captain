use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::Cancel;

#[test]
fn cancel_kills_the_running_child_and_blocks_the_next() {
    let cancel = Arc::new(Cancel::default());
    let runner = cancel.clone();
    let started = Instant::now();
    let run = std::thread::spawn(move || {
        let mut sleep = Command::new("sh");
        sleep.args(["-c", "sleep 30"]);
        runner.output(sleep, None)
    });
    std::thread::sleep(Duration::from_millis(200));
    cancel.cancel();
    assert!(run.join().unwrap().is_err());
    assert!(started.elapsed() < Duration::from_secs(10));
    assert!(cancel.output(Command::new("true"), None).is_err());
}

#[test]
fn cancel_kills_a_child_that_does_not_read_a_large_input() {
    let cancel = Arc::new(Cancel::default());
    let runner = cancel.clone();
    let started = Instant::now();
    let run = std::thread::spawn(move || {
        let mut sleep = Command::new("sleep");
        sleep.arg("30");
        // Larger than any pipe buffer, so a write before the kill would block.
        let input = "x".repeat(4 << 20);
        runner.output(sleep, Some(&input))
    });
    std::thread::sleep(Duration::from_millis(200));
    cancel.cancel();
    assert!(run.join().unwrap().is_err());
    assert!(started.elapsed() < Duration::from_secs(10));
}

use captain_core::migration::TransferEvent;
use futures::channel::mpsc;
use futures::executor::block_on;
use futures::{StreamExt, stream};

use super::{REPORT_EVERY, counted};

fn chunks(count: usize, size: usize) -> super::ByteStream {
    Box::pin(stream::iter((0..count).map(move |_| Ok(vec![0u8; size]))))
}

#[test]
fn reports_progress_and_passes_bytes_through() {
    let (tx, mut rx) = mpsc::unbounded();
    let size = REPORT_EVERY as usize;
    let out: Vec<_> = block_on(counted(chunks(3, size), 0, tx).collect());
    assert_eq!(out.len(), 3);
    assert!(
        out.iter()
            .all(|chunk| chunk.as_ref().is_ok_and(|c| c.len() == size))
    );
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event.expect("event"));
    }
    assert_eq!(
        events.first(),
        Some(&TransferEvent::Progress { done: 0, total: 0 })
    );
    let last = 3 * REPORT_EVERY;
    assert_eq!(
        events.last(),
        Some(&TransferEvent::Progress {
            done: last,
            total: last
        })
    );
}

#[test]
fn fails_once_the_receiver_is_gone() {
    let (tx, rx) = mpsc::unbounded();
    drop(rx);
    let out: Vec<_> = block_on(counted(chunks(1, 10), 10, tx).collect());
    assert!(out[0].is_err());
}

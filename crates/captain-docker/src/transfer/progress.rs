//! Progress reports for one copy, and the cancel signal that comes back: when the
//! UI drops the event stream, the next chunk fails and the copy stops.

use captain_core::EngineError;
use captain_core::migration::TransferEvent;
use futures::StreamExt;
use futures::channel::mpsc::{self, UnboundedSender};

use super::source::ByteStream;

/// The sending half of a copy's event stream.
pub type Events = UnboundedSender<Result<TransferEvent, EngineError>>;

/// How a copy of one item ended, when it did not fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Copied,
    /// Not copied, for the reason given.
    Skipped(String),
}

/// Report progress after about this many bytes.
const REPORT_EVERY: u64 = 1024 * 1024;

/// The error a copy stops with when the user cancels.
pub fn cancelled() -> EngineError {
    EngineError::Api("stopped by the user".into())
}

/// True once the UI dropped the event stream.
pub fn is_cancelled(events: &Events) -> bool {
    events.is_closed()
}

pub fn send(events: &Events, event: TransferEvent) {
    events.unbounded_send(Ok(event)).ok();
}

/// A sender that passes events on to `events` but never reads as cancelled. A
/// switch-over uses it once the source is stopped, so closing the assistant does
/// not leave the item stopped in the source and not started in the target. Call it
/// on the tokio runtime.
pub fn uncancellable(events: &Events) -> Events {
    let (sender, mut receiver) = mpsc::unbounded();
    let events = events.clone();
    tokio::spawn(async move {
        while let Some(event) = receiver.next().await {
            events.unbounded_send(event).ok();
        }
    });
    sender
}

/// Passes `stream` through, counting bytes and reporting progress against `total`.
/// It fails with [`cancelled`] once the UI is gone.
pub fn counted(stream: ByteStream, total: u64, events: Events) -> ByteStream {
    let mut done = 0u64;
    let mut reported = 0u64;
    send(&events, TransferEvent::Progress { done, total });
    Box::pin(stream.map(move |chunk| {
        let chunk = chunk?;
        if is_cancelled(&events) {
            return Err(cancelled());
        }
        done += chunk.len() as u64;
        if done - reported >= REPORT_EVERY {
            reported = done;
            let total = total.max(done);
            send(&events, TransferEvent::Progress { done, total });
        }
        Ok(chunk)
    }))
}

#[cfg(test)]
mod tests;

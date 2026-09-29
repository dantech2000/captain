//! Runs blocking work on a plain thread and hands the result back through a
//! runtime-neutral future.

use captain_core::{HostError, HostFuture};
use futures::FutureExt;
use futures::channel::oneshot;

/// Runs `work` on a new thread. Any executor can await the returned future.
pub fn blocking<T, F>(work: F) -> HostFuture<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, HostError> + Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || tx.send(work()).ok());
    rx.map(|result| result.unwrap_or_else(|_| Err(HostError("The host thread stopped.".into()))))
        .boxed()
}

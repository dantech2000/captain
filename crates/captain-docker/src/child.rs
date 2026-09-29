//! A child process that belongs to the stream of its output. Dropping the stream
//! kills and reaps the process, even while it prints nothing.

use std::io;
use std::pin::Pin;
use std::process::{Child, ExitStatus};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{Context, Poll};
use std::time::Duration;

use futures::{Stream, StreamExt};

/// How often [`SharedChild::wait`] looks whether the process has exited.
const POLL: Duration = Duration::from_millis(20);

/// A child that a waiting thread and a [`Guarded`] stream share.
#[derive(Clone)]
pub struct SharedChild(Arc<Mutex<Child>>);

impl SharedChild {
    pub fn new(child: Child) -> Self {
        Self(Arc::new(Mutex::new(child)))
    }

    fn lock(&self) -> MutexGuard<'_, Child> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Waits for the process to exit. The lock is free between checks, so a drop of
    /// the stream can kill the process meanwhile.
    pub fn wait(&self) -> io::Result<ExitStatus> {
        loop {
            if let Some(status) = self.lock().try_wait()? {
                return Ok(status);
            }
            std::thread::sleep(POLL);
        }
    }

    fn kill(&self) {
        let mut child = self.lock();
        if matches!(child.try_wait(), Ok(None)) {
            child.kill().ok();
            child.wait().ok();
        }
    }
}

/// A stream that kills and reaps its process when it drops.
pub struct Guarded<S> {
    stream: S,
    child: SharedChild,
}

impl<S> Guarded<S> {
    pub fn new(stream: S, child: SharedChild) -> Self {
        Self { stream, child }
    }
}

impl<S: Stream + Unpin> Stream for Guarded<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx)
    }
}

impl<S> Drop for Guarded<S> {
    fn drop(&mut self) {
        self.child.kill();
    }
}

//! A child process that belongs to the stream of its output. Dropping the stream
//! kills and reaps the process, even while it prints nothing.

use std::io::{self, Write};
use std::pin::Pin;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use futures::channel::oneshot;
use futures::future::BoxFuture;
use futures::{FutureExt, Stream, StreamExt};

use crate::process::drain;
use crate::process_group::{self, own_group};

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
        self.wait_until(None)
    }

    /// Like [`Self::wait`]. After `deadline`, kills and reaps the process and fails.
    fn wait_until(&self, deadline: Option<Instant>) -> io::Result<ExitStatus> {
        loop {
            if let Some(status) = self.lock().try_wait()? {
                return Ok(status);
            }
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                self.kill();
                return Err(io::Error::new(io::ErrorKind::TimedOut, "did not answer"));
            }
            std::thread::sleep(POLL);
        }
    }

    fn kill(&self) {
        let mut child = self.lock();
        if matches!(child.try_wait(), Ok(None)) {
            process_group::kill(&mut child);
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

/// Runs `command` to the end on a plain thread, like [`Command::output`]. Dropping
/// the future before the end kills and reaps the process.
pub fn output_guarded(command: Command) -> BoxFuture<'static, io::Result<Output>> {
    input_output_guarded(command, None, None)
}

/// Like [`output_guarded`], and writes `input` to the command's stdin on its own
/// thread, so a command that never reads cannot block. After `timeout`, the
/// process is killed and the future fails.
pub fn input_output_guarded(
    mut command: Command,
    input: Option<Vec<u8>>,
    timeout: Option<Duration>,
) -> BoxFuture<'static, io::Result<Output>> {
    let stdin = if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    };
    let spawned = own_group(&mut command)
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => return futures::future::ready(Err(error)).boxed(),
    };
    let deadline = timeout.map(|timeout| Instant::now() + timeout);
    if let (Some(input), Some(mut pipe)) = (input, child.stdin.take()) {
        std::thread::spawn(move || pipe.write_all(&input).ok());
    }
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let child = SharedChild::new(child);
    let waiter = child.clone();
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || {
        let output = waiter.wait_until(deadline).map(|status| Output {
            status,
            stdout: stdout.join().unwrap_or_default(),
            stderr: stderr.join().unwrap_or_default(),
        });
        tx.send(output).ok();
    });
    Guarded::new(rx.into_stream(), child)
        .into_future()
        .map(|(output, _guard)| match output {
            Some(Ok(output)) => output,
            _ => Err(io::Error::other("the command stopped")),
        })
        .boxed()
}

#[cfg(all(test, unix))]
mod tests;

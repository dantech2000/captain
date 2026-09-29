//! Runs bollard futures on a private tokio runtime and hands results back through
//! runtime-neutral futures and channels. See docs/adr/0002-bollard-and-tokio-bridge.md.

use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};

use captain_core::EngineError;
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::{FutureExt, Stream, StreamExt};
use tokio::runtime::{Builder, Handle, Runtime};
use tokio::task::AbortHandle;

pub fn build() -> std::io::Result<Runtime> {
    Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("captain-docker")
        .enable_all()
        .build()
}

/// A runtime that shuts down without waiting when it drops. A plain [`Runtime`]
/// waits for its `spawn_blocking` work, which would freeze the UI thread that drops
/// an engine. See <https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html#method.shutdown_background>.
pub struct BackgroundRuntime(Option<Runtime>);

impl From<Runtime> for BackgroundRuntime {
    fn from(runtime: Runtime) -> Self {
        Self(Some(runtime))
    }
}

impl Deref for BackgroundRuntime {
    type Target = Runtime;

    fn deref(&self) -> &Runtime {
        self.0.as_ref().expect("the runtime is set until drop")
    }
}

impl Drop for BackgroundRuntime {
    fn drop(&mut self) {
        if let Some(runtime) = self.0.take() {
            runtime.shutdown_background();
        }
    }
}

/// Spawns `future` on tokio. Any executor can await the returned future.
pub fn spawn<T, F>(handle: &Handle, future: F) -> BoxFuture<'static, Result<T, EngineError>>
where
    T: Send + 'static,
    F: Future<Output = Result<T, EngineError>> + Send + 'static,
{
    handle
        .spawn(future)
        .map(|joined| joined.unwrap_or_else(|err| Err(EngineError::Api(err.to_string()))))
        .boxed()
}

/// Runs `produce` on tokio with the sending half of a channel, and returns the
/// receiving half. Dropping the returned stream aborts `produce`, even while it
/// waits for the engine.
pub fn forward<T, F, Fut>(handle: &Handle, produce: F) -> BoxStream<'static, T>
where
    T: Send + 'static,
    F: FnOnce(UnboundedSender<T>) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (tx, rx) = mpsc::unbounded();
    let task = handle.spawn(produce(tx)).abort_handle();
    AbortOnDrop { rx, task }.boxed()
}

/// Like [`forward`], but `produce` runs to its end after the stream drops. For work
/// that must not stop halfway, such as a copy between engines.
pub fn forward_to_end<T, F, Fut>(handle: &Handle, produce: F) -> BoxStream<'static, T>
where
    T: Send + 'static,
    F: FnOnce(UnboundedSender<T>) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (tx, rx) = mpsc::unbounded();
    handle.spawn(produce(tx));
    rx.boxed()
}

/// A channel receiver that aborts its producer task when it drops.
struct AbortOnDrop<T> {
    rx: UnboundedReceiver<T>,
    task: AbortHandle,
}

impl<T> Stream for AbortOnDrop<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.rx.poll_next_unpin(cx)
    }
}

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[cfg(test)]
mod tests;

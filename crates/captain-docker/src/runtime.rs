//! Runs bollard futures on a private tokio runtime and hands results back through
//! runtime-neutral futures and channels. See docs/adr/0002-bollard-and-tokio-bridge.md.

use std::future::Future;

use captain_core::EngineError;
use futures::channel::mpsc::{self, UnboundedSender};
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::{FutureExt, StreamExt};
use tokio::runtime::{Builder, Handle, Runtime};

pub fn build() -> std::io::Result<Runtime> {
    Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("captain-docker")
        .enable_all()
        .build()
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
/// receiving half. `produce` should stop when a send fails, because that means the
/// receiver was dropped.
pub fn forward<T, F, Fut>(handle: &Handle, produce: F) -> BoxStream<'static, T>
where
    T: Send + 'static,
    F: FnOnce(UnboundedSender<T>) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (tx, rx) = mpsc::unbounded();
    handle.spawn(produce(tx));
    rx.boxed()
}

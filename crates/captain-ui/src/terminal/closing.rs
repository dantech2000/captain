use std::sync::mpsc;
use std::thread;
use std::time::Instant;

/// A session that ends on a thread of its own, for example a local shell that gets
/// a hangup and a kill. Quit waits for it; dropping it lets the end go on alone.
pub struct Closing(mpsc::Receiver<()>);

impl Closing {
    pub fn spawn(work: impl FnOnce() + Send + 'static) -> Self {
        let (done_tx, done_rx) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("terminal-end".into())
            .spawn(move || {
                work();
                done_tx.send(()).ok();
            });
        if let Err(error) = spawned {
            tracing::warn!(%error, "cannot start a terminal thread");
        }
        Self(done_rx)
    }

    /// Waits for the end, until `deadline` at most.
    pub fn wait_until(&self, deadline: Instant) {
        let left = deadline.saturating_duration_since(Instant::now());
        self.0.recv_timeout(left).ok();
    }
}

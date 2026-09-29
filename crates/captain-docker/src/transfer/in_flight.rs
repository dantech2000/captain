//! Counts the copies that run, so the end of a session can wait for a stopped copy
//! to remove its helpers and its half-copied volume before the runtime goes away.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// How often [`InFlight::wait_idle`] looks again.
const POLL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Default)]
pub struct InFlight(Arc<AtomicUsize>);

/// One running copy. Dropping it ends the copy's count.
pub struct Guard(Arc<AtomicUsize>);

impl InFlight {
    pub fn enter(&self) -> Guard {
        self.0.fetch_add(1, Ordering::SeqCst);
        Guard(self.0.clone())
    }

    pub fn count(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }

    /// Waits until no copy runs, or `limit` passes.
    pub async fn wait_idle(&self, limit: Duration) {
        let waited = tokio::time::timeout(limit, async {
            while self.count() > 0 {
                tokio::time::sleep(POLL).await;
            }
        });
        if waited.await.is_err() {
            tracing::warn!(
                copies = self.count(),
                "copies still run at the end of a session"
            );
        }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests;

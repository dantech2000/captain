//! Counts the copies that run, so the end of a session can wait for a stopped copy
//! to remove its helpers and its half-copied volume before the runtime goes away. A
//! session counts its switch-overs apart, because it waits for those without a
//! limit.

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

    /// Waits until no copy runs, or `limit` passes. With no limit, it waits as
    /// long as it takes. Returns true if no copy runs.
    pub async fn wait_idle(&self, limit: Option<Duration>) -> bool {
        let idle = async {
            while self.count() > 0 {
                tokio::time::sleep(POLL).await;
            }
        };
        match limit {
            Some(limit) => tokio::time::timeout(limit, idle).await.is_ok(),
            None => {
                idle.await;
                true
            }
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

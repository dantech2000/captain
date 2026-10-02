use std::sync::{Arc, Condvar, Mutex};

/// Caps the output bytes that wait between the reader thread and the view. A full
/// queue makes the reader wait, so the shell's writes block in the PTY instead of
/// filling Captain's memory. Cloning it gives another handle.
#[derive(Clone)]
pub struct OutputBudget(Arc<Inner>);

struct Inner {
    limit: usize,
    state: Mutex<State>,
    changed: Condvar,
}

#[derive(Default)]
struct State {
    queued: usize,
    /// The view stopped reading.
    closed: bool,
}

/// Gives back room as the view takes output, and closes the budget when dropped.
pub struct BudgetRelease(OutputBudget);

impl OutputBudget {
    pub fn new(limit: usize) -> Self {
        Self(Arc::new(Inner {
            limit,
            state: Mutex::new(State::default()),
            changed: Condvar::new(),
        }))
    }

    /// Takes room for `bytes`, and waits while the queue is full. An empty queue
    /// takes any chunk. False once the view stopped reading.
    pub fn acquire(&self, bytes: usize) -> bool {
        let Ok(mut state) = self.0.state.lock() else {
            return false;
        };
        loop {
            if state.closed {
                return false;
            }
            if state.queued == 0 || state.queued + bytes <= self.0.limit {
                state.queued += bytes;
                return true;
            }
            state = match self.0.changed.wait(state) {
                Ok(state) => state,
                Err(_) => return false,
            };
        }
    }

    /// The handle for the view's side.
    pub fn release_handle(&self) -> BudgetRelease {
        BudgetRelease(self.clone())
    }

    fn update(&self, change: impl FnOnce(&mut State)) {
        if let Ok(mut state) = self.0.state.lock() {
            change(&mut state);
            self.0.changed.notify_all();
        }
    }
}

impl BudgetRelease {
    /// The view took `bytes` off the queue.
    pub fn release(&self, bytes: usize) {
        self.0
            .update(|state| state.queued = state.queued.saturating_sub(bytes));
    }
}

impl Drop for BudgetRelease {
    fn drop(&mut self) {
        self.0.update(|state| state.closed = true);
    }
}

#[cfg(test)]
mod tests;

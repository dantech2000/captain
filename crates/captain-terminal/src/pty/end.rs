//! Ending a PTY session: hang up, wait, kill what is left, and wait again.

use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use super::PtyControl;

/// How long killed processes get to go away.
const REAP_WAIT: Duration = Duration::from_millis(500);
/// How often the end checks whether the processes have gone.
const TICK: Duration = Duration::from_millis(20);

impl PtyControl {
    /// Ends the session and returns once its processes have gone, or after
    /// `grace` and half a second at most. On Unix it sends `SIGHUP` to the shell's
    /// process group and to the PTY's foreground group, as a closed terminal window
    /// does. Groups still there after `grace` get `SIGKILL`. On Windows it ends the
    /// shell at once; closing the pseudoconsole ends the programs attached to it.
    /// Then the reader stops and writes fail. A shell that already exited gets no
    /// signals. A second call waits for the first and returns.
    pub fn terminate(&self, grace: Duration) {
        let Ok(mut ended) = self.state.ended.lock() else {
            return;
        };
        if *ended {
            return;
        }
        *ended = true;
        if !self.exited() {
            self.end_processes(grace);
        }
        self.state.closed.store(true, Ordering::SeqCst);
        if let Ok(mut master) = self.master.lock() {
            master.take();
        }
    }

    #[cfg(unix)]
    fn end_processes(&self, grace: Duration) {
        use rustix::process::Signal;
        // Taken before the hangup: once the shell exits, the PTY may no longer
        // name its foreground group.
        let groups = self.groups();
        signal(&groups, Signal::HUP);
        if self.wait_for(grace, || all_gone(&groups)) {
            return;
        }
        signal(&groups, Signal::KILL);
        self.kill_shell();
        self.wait_for(REAP_WAIT, || all_gone(&groups));
    }

    #[cfg(windows)]
    fn end_processes(&self, _grace: Duration) {
        self.kill_shell();
        self.wait_for(REAP_WAIT, || true);
    }

    /// The shell's process group and the PTY's foreground group.
    #[cfg(unix)]
    fn groups(&self) -> Vec<rustix::process::Pid> {
        use rustix::process::Pid;
        let foreground = self.master.lock().ok().and_then(|master| {
            master
                .as_ref()
                .and_then(|master| master.process_group_leader())
        });
        let shell = self.pid.and_then(|pid| i32::try_from(pid).ok());
        let mut groups: Vec<Pid> = [shell, foreground]
            .into_iter()
            .flatten()
            .filter_map(Pid::from_raw)
            .collect();
        groups.dedup();
        groups
    }

    fn kill_shell(&self) {
        if let Ok(mut killer) = self.killer.lock() {
            killer.kill().ok();
        }
    }

    fn exited(&self) -> bool {
        self.state.exited.load(Ordering::SeqCst)
    }

    /// Waits up to `limit` for the shell to be reaped and `others_gone`. True when
    /// both happened.
    fn wait_for(&self, limit: Duration, others_gone: impl Fn() -> bool) -> bool {
        let start = Instant::now();
        loop {
            if self.exited() && others_gone() {
                return true;
            }
            if start.elapsed() >= limit {
                return false;
            }
            thread::sleep(TICK);
        }
    }
}

#[cfg(unix)]
fn signal(groups: &[rustix::process::Pid], signal: rustix::process::Signal) {
    for group in groups {
        // Fails harmlessly when the group is gone.
        rustix::process::kill_process_group(*group, signal).ok();
    }
}

/// True when no process is left in any of `groups`.
#[cfg(unix)]
fn all_gone(groups: &[rustix::process::Pid]) -> bool {
    groups.iter().all(|group| {
        rustix::process::test_kill_process_group(*group) == Err(rustix::io::Errno::SRCH)
    })
}

#[cfg(all(test, unix))]
mod tests;

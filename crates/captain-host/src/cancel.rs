//! [`Cancel`]: the child process a host action runs now, and the flag that stops
//! the action. A stop kills the child and sets the flag, so the action ends at once
//! and does not run its next step. See docs/features/0013-captain-engine.md.

use std::io::{Read, Write};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;

use captain_core::HostError;

/// How often [`Cancel::output`] looks whether its child has exited.
const POLL: Duration = Duration::from_millis(25);

#[derive(Default)]
pub struct Cancel {
    running: Mutex<Option<Child>>,
    cancelled: AtomicBool,
}

impl Cancel {
    /// Clears the flag for a new action.
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }

    /// Sets the flag and kills the running child.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.kill();
    }

    /// Kills the running child, if any.
    pub fn kill(&self) {
        if let Some(child) = lock(&self.running).as_mut() {
            child.kill().ok();
        }
    }

    /// Fails once the action is cancelled.
    pub fn check(&self) -> Result<(), HostError> {
        if self.cancelled.load(Ordering::SeqCst) {
            return Err(cancelled());
        }
        Ok(())
    }

    /// Keeps `child` where [`Cancel::kill`] reaches it.
    pub fn hold(&self, child: Child) {
        *lock(&self.running) = Some(child);
        if self.cancelled.load(Ordering::SeqCst) {
            self.kill();
        }
    }

    /// Takes back the child that [`Cancel::hold`] kept.
    pub fn take(&self) -> Option<Child> {
        lock(&self.running).take()
    }

    /// Runs `command` with `input` on standard input, and returns its output. The
    /// child can be killed meanwhile, and a cancelled action fails without waiting
    /// for its pipes, which a grandchild such as `ssh` may still hold.
    pub fn output(&self, mut command: Command, input: Option<&str>) -> Result<Output, HostError> {
        self.check()?;
        let program = command.get_program().to_string_lossy().into_owned();
        let stdin = if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        };
        let mut child = command
            .stdin(stdin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| HostError(format!("Cannot run {program}: {error}")))?;
        let stdout = child.stdout.take().map(read_all);
        let stderr = child.stderr.take().map(read_all);
        let pipe = child.stdin.take();
        self.hold(child);
        // The input goes on its own thread, after the child is held, so a child
        // that does not read a large input can still be killed. The write ends
        // once no process holds the pipe.
        if let (Some(mut pipe), Some(input)) = (pipe, input) {
            let input = input.to_owned();
            std::thread::spawn(move || pipe.write_all(input.as_bytes()).ok());
        }
        let status = loop {
            let mut running = lock(&self.running);
            let Some(child) = running.as_mut() else {
                return Err(cancelled());
            };
            match child.try_wait() {
                Ok(Some(status)) => {
                    running.take();
                    break status;
                }
                Ok(None) => {}
                Err(error) => {
                    if let Some(mut child) = running.take() {
                        child.kill().ok();
                        child.wait().ok();
                    }
                    return Err(HostError(error.to_string()));
                }
            }
            drop(running);
            std::thread::sleep(POLL);
        };
        self.check()?;
        Ok(Output {
            status,
            stdout: join(stdout),
            stderr: join(stderr),
        })
    }
}

fn read_all(mut pipe: impl Read + Send + 'static) -> JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        pipe.read_to_end(&mut bytes).ok();
        bytes
    })
}

fn join(reader: Option<JoinHandle<Vec<u8>>>) -> Vec<u8> {
    reader
        .and_then(|reader| reader.join().ok())
        .unwrap_or_default()
}

fn cancelled() -> HostError {
    HostError("The start was cancelled.".into())
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(all(test, unix))]
mod tests;

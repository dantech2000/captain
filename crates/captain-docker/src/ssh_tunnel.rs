//! The SSH tunnel to a remote engine. Captain runs the system `ssh` to forward the
//! remote Docker socket to a local Unix socket, restarts it when it exits, and stops
//! it when Captain switches engines or quits. See
//! docs/features/0026-contexts-and-remote-hosts.md.

use std::ffi::OsString;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use captain_core::ssh::SshTarget;

mod slot;

use slot::{Register, Slot, close_in, open_in};

/// How long `ssh` may take to log in and open the local socket.
const READY_TIMEOUT: Duration = Duration::from_secs(40);
/// How often the supervisor checks that `ssh` still runs.
const POLL: Duration = Duration::from_millis(500);
/// The wait before the first restart. It doubles after each failed restart.
const FIRST_RESTART: Duration = Duration::from_secs(1);
const MAX_RESTART: Duration = Duration::from_secs(30);

/// The one tunnel Captain keeps open.
static ACTIVE: Mutex<Slot> = Mutex::new(Slot::new());
/// Numbers the tunnel directories of this process.
static NEXT_DIR: AtomicU32 = AtomicU32::new(0);

/// Opens a tunnel to `target`, or reuses the open one, and returns the local socket.
/// A tunnel to another host stops. This blocks until `ssh` logs in; call it from a
/// background thread. The error is a message for the user.
pub fn open_ssh_tunnel(target: &SshTarget) -> Result<PathBuf, String> {
    if !cfg!(unix) {
        return Err("Captain reaches engines over SSH on macOS and Linux only.".into());
    }
    open_in(&ACTIVE, target, |register| {
        let base = std::env::temp_dir();
        SshTunnel::start_with("ssh".into(), target.clone(), &base, register)
    })
}

/// Stops the open tunnel, if there is one. A start still in progress stops too, and
/// its `ssh` is killed before this returns, so a quit leaves no `ssh` behind.
pub fn close_ssh_tunnel() {
    close_in(&ACTIVE);
}

/// A running `ssh -L` and the thread that restarts it. Dropping it stops both and
/// removes the socket directory.
pub(crate) struct SshTunnel {
    target: SshTarget,
    dir: PathBuf,
    socket: PathBuf,
    shared: Arc<Shared>,
    supervisor: Option<JoinHandle<()>>,
}

/// What the tunnel and its supervisor share.
struct Shared {
    program: OsString,
    target: SshTarget,
    socket: PathBuf,
    child: Mutex<Option<Child>>,
    stopped: Mutex<bool>,
    wake: Condvar,
}

impl SshTunnel {
    /// Starts `program` (the `ssh` binary) with a socket in a new 0700 directory
    /// under `base`, and waits until the socket exists.
    #[cfg(all(test, unix))]
    pub(crate) fn start(program: OsString, target: SshTarget, base: &Path) -> Result<Self, String> {
        Self::start_with(program, target, base, None)
    }

    /// Like `start`, and hands `register` a way to stop the start while it waits.
    pub(crate) fn start_with(
        program: OsString,
        target: SshTarget,
        base: &Path,
        register: Option<Register<'_>>,
    ) -> Result<Self, String> {
        let number = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let dir = base.join(format!("captain-ssh-{}-{number}", std::process::id()));
        private_dir(&dir).map_err(|err| format!("cannot make {}: {err}", dir.display()))?;
        let socket = dir.join("docker.sock");
        let shared = Arc::new(Shared {
            program,
            target: target.clone(),
            socket: socket.clone(),
            child: Mutex::new(None),
            stopped: Mutex::new(false),
            wake: Condvar::new(),
        });
        let closed = register.is_some_and(|register| !register.starting(&shared));
        if closed {
            let _ = fs::remove_dir_all(&dir);
            return Err(format!("The connection to {target} was closed."));
        }
        if let Err(error) = shared.spawn_ready() {
            let _ = fs::remove_dir_all(&dir);
            return Err(error);
        }
        tracing::info!(%target, socket = %socket.display(), "SSH tunnel is up");
        let supervisor = {
            let shared = shared.clone();
            std::thread::spawn(move || shared.supervise())
        };
        Ok(Self {
            target,
            dir,
            socket,
            shared,
            supervisor: Some(supervisor),
        })
    }

    #[cfg(all(test, unix))]
    pub(crate) fn socket(&self) -> &Path {
        &self.socket
    }

    /// The process ID of the current `ssh`.
    #[cfg(all(test, unix))]
    pub(crate) fn pid(&self) -> Option<u32> {
        self.shared.lock_child().as_ref().map(Child::id)
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        self.shared.stop();
        if let Some(supervisor) = self.supervisor.take() {
            let _ = supervisor.join();
        }
        // The supervisor may have started one more `ssh` before it saw the stop.
        self.shared.stop();
        let _ = fs::remove_dir_all(&self.dir);
        tracing::info!(target = %self.target, "SSH tunnel stopped");
    }
}

impl Shared {
    fn lock_child(&self) -> std::sync::MutexGuard<'_, Option<Child>> {
        self.child.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Marks the tunnel stopped, wakes the supervisor, and kills `ssh`.
    fn stop(&self) {
        *self.stopped.lock().unwrap_or_else(PoisonError::into_inner) = true;
        self.wake.notify_all();
        if let Some(mut child) = self.lock_child().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn is_stopped(&self) -> bool {
        *self.stopped.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Waits up to `wait`, and returns true as soon as the tunnel is stopped.
    fn wait_stopped(&self, wait: Duration) -> bool {
        let stopped = self.stopped.lock().unwrap_or_else(PoisonError::into_inner);
        let (stopped, _) = self
            .wake
            .wait_timeout_while(stopped, wait, |stopped| !*stopped)
            .unwrap_or_else(PoisonError::into_inner);
        *stopped
    }

    /// Restarts `ssh` whenever it exits, until the tunnel is stopped.
    fn supervise(&self) {
        let mut delay = FIRST_RESTART;
        while !self.wait_stopped(POLL) {
            let exited = match self.lock_child().as_mut() {
                Some(child) => !matches!(child.try_wait(), Ok(None)),
                None => true,
            };
            if !exited {
                continue;
            }
            tracing::warn!(target = %self.target, "the SSH tunnel stopped; restarting in {delay:?}");
            if self.wait_stopped(delay) {
                break;
            }
            match self.spawn_ready() {
                Ok(()) => {
                    delay = FIRST_RESTART;
                    tracing::info!(target = %self.target, "SSH tunnel is up again");
                }
                Err(error) => {
                    delay = (delay * 2).min(MAX_RESTART);
                    tracing::warn!(%error, "cannot restart the SSH tunnel");
                }
            }
        }
    }

    /// Runs `ssh` and waits until it makes the local socket, which it does after it
    /// logs in. The error explains why `ssh` exited. The child waits in `child`, so
    /// [`Shared::stop`] can kill it at any time.
    fn spawn_ready(&self) -> Result<(), String> {
        let _ = fs::remove_file(&self.socket);
        let child = Command::new(&self.program)
            .args(self.target.tunnel_args(&self.socket))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| format!("Captain cannot run ssh: {err}"))?;
        *self.lock_child() = Some(child);
        let deadline = Instant::now() + READY_TIMEOUT;
        loop {
            let mut guard = self.lock_child();
            let Some(child) = guard.as_mut().filter(|_| !self.is_stopped()) else {
                drop(guard);
                self.stop();
                return Err(format!("The connection to {} was closed.", self.target));
            };
            if self.socket.exists() {
                log_stderr(child, self.target.to_string());
                return Ok(());
            }
            if let Ok(Some(_)) = child.try_wait() {
                let mut stderr = String::new();
                if let Some(mut pipe) = child.stderr.take() {
                    let _ = pipe.read_to_string(&mut stderr);
                }
                *guard = None;
                return Err(self.target.explain_failure(&stderr));
            }
            if Instant::now() > deadline {
                if let Some(mut child) = guard.take() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                return Err(format!(
                    "SSH to {} did not open the tunnel in time.",
                    self.target.host
                ));
            }
            drop(guard);
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

/// Logs what `ssh` writes after the tunnel is up, so its pipe never fills.
fn log_stderr(child: &mut Child, target: String) {
    if let Some(pipe) = child.stderr.take() {
        std::thread::spawn(move || {
            for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                tracing::warn!(%target, "ssh: {line}");
            }
        });
    }
}

/// Makes `dir` anew, readable only by the user.
fn private_dir(dir: &Path) -> std::io::Result<()> {
    let _ = fs::remove_dir_all(dir);
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(dir)
}

#[cfg(all(test, unix))]
mod tests;

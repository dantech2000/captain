//! A shell in a pseudo-terminal, for the terminal panel. `portable-pty` opens the
//! PTY: `openpty` on macOS and Linux, ConPTY on Windows. It gives a blocking reader
//! and writer for background threads, a resize that returns an error instead of
//! exiting, and a child to wait on. See docs/features/0041-integrated-terminal.md.

use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use portable_pty::{ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};

/// What to run in the PTY, and where.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShellCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    /// Variables to set on top of Captain's own environment.
    pub env: Vec<(String, OsString)>,
    /// Variables to remove from it.
    pub env_remove: Vec<String>,
}

/// A running shell: its output, its input, the control handle, and the child.
pub struct Pty {
    pub reader: Box<dyn Read + Send>,
    /// Dropping it sends end of file to the shell, so hang up the shell first.
    pub writer: Box<dyn Write + Send>,
    pub control: PtyControl,
    pub child: PtyChild,
}

/// Resizes the PTY and ends its processes. Cloning it gives another handle.
#[derive(Clone)]
pub struct PtyControl {
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    killer: Arc<Mutex<Box<dyn ChildKiller + Send + Sync>>>,
    /// The shell's process ID, which is also its process group's.
    #[cfg(unix)]
    pid: Option<u32>,
}

/// The shell process.
pub struct PtyChild(Box<dyn portable_pty::Child + Send + Sync>);

/// Starts `command` in a new PTY of `cols` by `rows`. On Unix the shell leads a
/// session of its own, with the PTY as its controlling terminal.
pub fn spawn(command: &ShellCommand, cols: u16, rows: u16) -> io::Result<Pty> {
    let size = PtySize {
        rows,
        cols,
        ..PtySize::default()
    };
    let pair = native_pty_system().openpty(size).map_err(other)?;
    let mut builder = CommandBuilder::new(&command.program);
    builder.args(&command.args);
    builder.cwd(&command.cwd);
    for key in &command.env_remove {
        builder.env_remove(key);
    }
    for (key, value) in &command.env {
        builder.env(key, value);
    }
    let child = pair.slave.spawn_command(builder).map_err(other)?;
    // The shell holds its own copy of the PTY's other end. Dropping this one lets
    // the reader see end of file once the shell and its jobs are gone.
    drop(pair.slave);
    let reader = pair.master.try_clone_reader().map_err(other)?;
    let writer = pair.master.take_writer().map_err(other)?;
    let control = PtyControl {
        killer: Arc::new(Mutex::new(child.clone_killer())),
        #[cfg(unix)]
        pid: child.process_id(),
        master: Arc::new(Mutex::new(pair.master)),
    };
    Ok(Pty {
        reader,
        writer,
        control,
        child: PtyChild(child),
    })
}

impl PtyControl {
    /// Tells the PTY (`TIOCSWINSZ`, or `ResizePseudoConsole` on Windows) that the
    /// terminal is now `cols` by `rows`. The shell gets `SIGWINCH`.
    pub fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        let size = PtySize {
            rows,
            cols,
            ..PtySize::default()
        };
        let master = self
            .master
            .lock()
            .map_err(|_| other("the PTY lock is poisoned"))?;
        master.resize(size).map_err(other)
    }

    /// Asks the shell and the program in the foreground to end: `SIGHUP` to both
    /// process groups, as when a terminal window closes. On Windows it ends the
    /// shell; closing the pseudoconsole ends the programs attached to it.
    pub fn hangup(&self) {
        #[cfg(unix)]
        self.signal_groups(rustix::process::Signal::HUP);
        #[cfg(windows)]
        self.kill();
    }

    /// Ends the shell and the foreground program at once: `SIGKILL` to both process
    /// groups on Unix, `TerminateProcess` on Windows.
    pub fn kill(&self) {
        #[cfg(unix)]
        self.signal_groups(rustix::process::Signal::KILL);
        if let Ok(mut killer) = self.killer.lock() {
            killer.kill().ok();
        }
    }

    /// Sends `signal` to the shell's process group and to the PTY's foreground
    /// group, which job control gives each program the shell starts.
    #[cfg(unix)]
    fn signal_groups(&self, signal: rustix::process::Signal) {
        use rustix::process::{Pid, kill_process_group};
        let foreground = self
            .master
            .lock()
            .ok()
            .and_then(|master| master.process_group_leader());
        let shell = self.pid.and_then(|pid| i32::try_from(pid).ok());
        for group in [foreground, shell].into_iter().flatten() {
            if let Some(pid) = Pid::from_raw(group) {
                // Fails harmlessly when the group is gone.
                kill_process_group(pid, signal).ok();
            }
        }
    }
}

impl PtyChild {
    /// Waits for the shell to exit and returns its exit code. A shell that a signal
    /// ended reports 1, as `portable-pty` maps it.
    pub fn wait(mut self) -> Option<i64> {
        self.0
            .wait()
            .ok()
            .map(|status| i64::from(status.exit_code()))
    }
}

fn other(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}

#[cfg(all(test, unix))]
mod tests;

//! A shell in a pseudo-terminal, for the terminal panel. `portable-pty` opens the
//! PTY: `openpty` on macOS and Linux, ConPTY on Windows. It gives a resize that
//! returns an error instead of exiting, and a child to wait on. On Unix the reader
//! and writer are Captain's own non-blocking ones, so ending the session can stop
//! them. See docs/features/0041-integrated-terminal.md.

use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use portable_pty::{ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};

mod end;
#[cfg(unix)]
mod unix_io;

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
    /// The shell's output. It ends once [`PtyControl::terminate`] has run.
    pub reader: Box<dyn Read + Send>,
    /// The shell's input. Writes fail once [`PtyControl::terminate`] has run, even
    /// one that waits for room.
    pub writer: Box<dyn Write + Send>,
    pub control: PtyControl,
    pub child: PtyChild,
}

/// Resizes the PTY and ends its processes. Cloning it gives another handle.
#[derive(Clone)]
pub struct PtyControl {
    /// `None` once the session has ended.
    master: Arc<Mutex<Option<Box<dyn MasterPty + Send>>>>,
    killer: Arc<Mutex<Box<dyn ChildKiller + Send + Sync>>>,
    state: Arc<State>,
    /// The shell's process ID, which is also its process group's.
    #[cfg(unix)]
    pid: Option<u32>,
}

/// What the control, the child, and the reader and writer share.
#[derive(Default)]
struct State {
    /// The shell has exited and has been reaped.
    exited: AtomicBool,
    /// The session has ended: the reader stops and writes fail.
    closed: Arc<AtomicBool>,
    /// [`PtyControl::terminate`] has run, or runs now.
    ended: Mutex<bool>,
}

/// The shell process.
pub struct PtyChild {
    child: Box<dyn portable_pty::Child + Send + Sync>,
    state: Arc<State>,
}

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
    let state = Arc::new(State::default());
    #[cfg(unix)]
    let (reader, writer) = unix_io::reader_writer(pair.master.as_ref(), state.closed.clone())?;
    // On Windows, closing the pseudoconsole at the end breaks both pipes.
    #[cfg(windows)]
    let (reader, writer) = (
        pair.master.try_clone_reader().map_err(other)?,
        pair.master.take_writer().map_err(other)?,
    );
    let control = PtyControl {
        killer: Arc::new(Mutex::new(child.clone_killer())),
        #[cfg(unix)]
        pid: child.process_id(),
        master: Arc::new(Mutex::new(Some(pair.master))),
        state: state.clone(),
    };
    Ok(Pty {
        reader,
        writer,
        control,
        child: PtyChild { child, state },
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
        let master = master
            .as_ref()
            .ok_or_else(|| other("the terminal session has ended"))?;
        master.resize(size).map_err(other)
    }
}

impl PtyChild {
    /// Waits for the shell to exit and returns its exit code. A shell that a signal
    /// ended reports 1, as `portable-pty` maps it.
    pub fn wait(mut self) -> Option<i64> {
        let status = self.child.wait();
        self.state.exited.store(true, Ordering::SeqCst);
        status.ok().map(|status| i64::from(status.exit_code()))
    }
}

fn other(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}

#[cfg(all(test, unix))]
mod tests;

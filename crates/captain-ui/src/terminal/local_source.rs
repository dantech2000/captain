use std::cell::RefCell;

use captain_core::model::ExecSession;
use captain_core::{EngineError, EngineFuture};
use captain_terminal::{PtyControl, ShellCommand, spawn_pty};
use futures::FutureExt;
use futures::future::ready;

use super::TerminalSource;
use super::pty_session::pty_session;

/// A shell on this computer in a PTY, for the terminal panel. Each session starts
/// with `banner`, a dim line that says which engine `docker` uses.
pub struct LocalSource {
    command: ShellCommand,
    banner: String,
    /// The running shell, to hang up on close.
    control: RefCell<Option<PtyControl>>,
}

impl LocalSource {
    pub fn new(command: ShellCommand, banner: String) -> Self {
        Self {
            command,
            banner,
            control: RefCell::new(None),
        }
    }
}

impl TerminalSource for LocalSource {
    fn open(&self, cols: u16, rows: u16) -> EngineFuture<ExecSession> {
        let program = self.command.program.display().to_string();
        let pty = match spawn_pty(&self.command, cols, rows) {
            Ok(pty) => pty,
            Err(error) => {
                return ready(Err(EngineError::Api(format!("{program}: {error}")))).boxed();
            }
        };
        self.control.replace(Some(pty.control.clone()));
        let mut command = vec![program];
        command.extend(self.command.args.iter().cloned());
        let banner = format!("\x1b[2m{}\x1b[0m\r\n", self.banner);
        ready(Ok(pty_session(pty, banner, command))).boxed()
    }

    fn close(&self) {
        if let Some(control) = self.control.take() {
            control.hangup();
        }
    }
}

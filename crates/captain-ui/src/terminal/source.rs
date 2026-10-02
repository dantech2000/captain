use captain_core::EngineFuture;
use captain_core::model::ExecSession;

use super::Closing;

/// Where a terminal's session comes from: an exec in a container, or a shell on
/// this computer. Both give an [`ExecSession`]: input, raw output, a resizer, and
/// the exit code.
pub trait TerminalSource: 'static {
    /// Starts a session with a TTY of `cols` by `rows`.
    fn open(&self, cols: u16, rows: u16) -> EngineFuture<ExecSession>;

    /// Starts to end the processes of the running session, for a closed tab or
    /// Quit. Dropping the session does the rest. Quit waits for the returned end.
    fn close(&self) -> Option<Closing> {
        None
    }
}

use captain_core::EngineFuture;
use captain_core::model::ExecSession;

/// Where a terminal's session comes from: an exec in a container, or a shell on
/// this computer. Both give an [`ExecSession`]: input, raw output, a resizer, and
/// the exit code.
pub trait TerminalSource: 'static {
    /// Starts a session with a TTY of `cols` by `rows`.
    fn open(&self, cols: u16, rows: u16) -> EngineFuture<ExecSession>;

    /// Ends the processes of the running session at once, for a closed tab or Quit.
    /// Dropping the session does the rest.
    fn close(&self) {}
}

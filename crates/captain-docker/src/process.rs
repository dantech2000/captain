//! Runs short CLI commands with a time limit.

use std::process::{Command, Output};
use std::sync::mpsc;
use std::time::Duration;

/// Runs `command` and waits at most `timeout`. A command that hangs keeps its
/// thread, which is fine for a one-off check.
pub fn output_within(mut command: Command, timeout: Duration) -> Result<Output, String> {
    let program = command.get_program().to_string_lossy().into_owned();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || tx.send(command.output()).ok());
    match rx.recv_timeout(timeout) {
        Ok(result) => result.map_err(|err| format!("cannot run {program}: {err}")),
        Err(_) => Err(format!("{program} did not answer")),
    }
}

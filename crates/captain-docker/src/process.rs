//! Runs short CLI commands with a time limit.

use std::io::{Read, Write};
use std::process::{Child, Command, Output, Stdio};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::process_group::{self, own_group};

/// How often to look whether the command has exited.
const POLL: Duration = Duration::from_millis(10);

/// Runs `command` and waits at most `timeout`. A command that takes longer is
/// killed and reaped.
pub fn output_within(command: Command, timeout: Duration) -> Result<Output, String> {
    input_output_within(command, None, timeout)
}

/// Like [`output_within`], and writes `input` to the command's stdin first.
pub fn input_output_within(
    mut command: Command,
    input: Option<Vec<u8>>,
    timeout: Duration,
) -> Result<Output, String> {
    let program = command.get_program().to_string_lossy().into_owned();
    let stdin = if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    };
    let mut child = own_group(&mut command)
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("cannot run {program}: {err}"))?;
    if let (Some(input), Some(mut pipe)) = (input, child.stdin.take()) {
        // On its own thread, so a command that never reads cannot block the timeout.
        std::thread::spawn(move || pipe.write_all(&input).ok());
    }
    // Both pipes drain while the command runs, so a full pipe cannot stall it.
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let status = wait_within(&mut child, timeout).map_err(|why| format!("{program} {why}"))?;
    Ok(Output {
        status,
        stdout: stdout.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
    })
}

/// Waits for `child` to exit. After `timeout`, kills and reaps it.
fn wait_within(child: &mut Child, timeout: Duration) -> Result<std::process::ExitStatus, String> {
    let deadline = Instant::now() + timeout;
    loop {
        let why = match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(POLL);
                continue;
            }
            Ok(None) => "did not answer".to_string(),
            Err(err) => format!("failed: {err}"),
        };
        process_group::kill(child);
        child.wait().ok();
        return Err(why);
    }
}

/// Reads all of `pipe` on its own thread.
pub(crate) fn drain(pipe: Option<impl Read + Send + 'static>) -> JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = pipe {
            pipe.read_to_end(&mut bytes).ok();
        }
        bytes
    })
}

#[cfg(all(test, unix))]
mod tests;

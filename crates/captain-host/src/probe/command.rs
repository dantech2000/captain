use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

/// How often to look whether the command has exited.
const POLL: Duration = Duration::from_millis(25);

/// Runs `command` and waits at most `timeout`. A command that takes longer is
/// killed. Only for commands that print little: the output is read after exit.
pub fn output_within(mut command: Command, timeout: Duration) -> Result<Output, String> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().map_err(|error| error.to_string()),
            Ok(None) if started.elapsed() < timeout => std::thread::sleep(POLL),
            Ok(None) => {
                child.kill().ok();
                child.wait().ok();
                return Err(format!(
                    "It did not answer within {} seconds.",
                    timeout.as_secs()
                ));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

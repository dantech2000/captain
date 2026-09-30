//! Runs a short helper command with a time limit.

use std::io::Read;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// How long Captain waits for the pipes after the command exits. A program the
/// command started in the background can keep them open.
const DRAIN: Duration = Duration::from_secs(1);

/// Runs `command` without input and returns its output, or an error when it cannot
/// start or runs longer than `timeout` (then it is killed).
pub fn output_within(mut command: Command, timeout: Duration) -> Result<Output, String> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    // Read the pipes while waiting, so a full pipe never blocks the child.
    let (sender, receiver) = mpsc::channel();
    let pipes: [Option<Box<dyn Read + Send>>; 2] = [
        child.stdout.take().map(|pipe| Box::new(pipe) as _),
        child.stderr.take().map(|pipe| Box::new(pipe) as _),
    ];
    for (ix, pipe) in pipes.into_iter().enumerate() {
        let Some(mut pipe) = pipe else { continue };
        let sender = sender.clone();
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = pipe.read_to_end(&mut bytes);
            let _ = sender.send((ix, bytes));
        });
    }
    drop(sender);
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("it took longer than {} s", timeout.as_secs()));
            }
            Err(error) => return Err(error.to_string()),
        }
    };
    let mut output = [Vec::new(), Vec::new()];
    while let Ok((ix, bytes)) = receiver.recv_timeout(DRAIN) {
        output[ix] = bytes;
    }
    let [stdout, stderr] = output;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

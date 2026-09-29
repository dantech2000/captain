//! Runs `limactl`, always with Captain's own `LIMA_HOME`, so it never sees or
//! changes the user's Lima VMs.

use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use captain_core::HostError;

use super::args::progress_line;
use crate::cancel::Cancel;
use crate::probe::output_within;

/// A `limactl` binary and the `LIMA_HOME` it runs with.
#[derive(Debug, Clone)]
pub struct Limactl {
    binary: PathBuf,
    lima_home: PathBuf,
}

impl Limactl {
    pub fn new(binary: PathBuf, lima_home: PathBuf) -> Self {
        Self { binary, lima_home }
    }

    pub fn binary(&self) -> &Path {
        &self.binary
    }

    /// A command that shares the caller's standard input, output, and error.
    pub fn interactive(&self, args: &[String]) -> Result<Command, HostError> {
        let mut command = self.command(args)?;
        command.stdin(Stdio::inherit());
        Ok(command)
    }

    /// The only way this crate builds a `limactl` command: `LIMA_HOME` is always set.
    fn command(&self, args: &[String]) -> Result<Command, HostError> {
        std::fs::create_dir_all(&self.lima_home).map_err(|error| {
            HostError(format!(
                "Cannot create {}: {error}",
                self.lima_home.display()
            ))
        })?;
        let mut command = Command::new(&self.binary);
        command
            .args(args)
            .env("LIMA_HOME", &self.lima_home)
            .env_remove("LIMA_INSTANCE")
            .stdin(Stdio::null());
        Ok(command)
    }

    /// Runs `args` and returns its standard output. Blocks until it exits.
    pub fn output(&self, args: &[String]) -> Result<String, HostError> {
        let output = self
            .command(args)?
            .output()
            .map_err(|error| spawn_error(&self.binary, error))?;
        finish(output)
    }

    /// Like [`Limactl::output`], but it gives up after `timeout`, for the quick
    /// status checks that must not hang the status.
    pub fn output_within(&self, args: &[String], timeout: Duration) -> Result<String, HostError> {
        let output = output_within(self.command(args)?, timeout)
            .map_err(|error| HostError(format!("limactl {}: {error}", args.join(" "))))?;
        finish(output)
    }

    /// Like [`Limactl::output`], but `cancel` can kill it, with `input` on standard
    /// input.
    pub fn run(
        &self,
        args: &[String],
        input: Option<&str>,
        cancel: &Cancel,
    ) -> Result<String, HostError> {
        finish(cancel.output(self.command(args)?, input)?)
    }

    /// Runs `args`, passes each progress line to `sink`, and blocks until it exits.
    /// The child waits in `cancel`, so another thread can kill it.
    pub fn stream(
        &self,
        args: &[String],
        cancel: &Cancel,
        sink: &mut dyn FnMut(String),
    ) -> Result<(), HostError> {
        cancel.check()?;
        let mut child = self
            .command(args)?
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| spawn_error(&self.binary, error))?;
        let (tx, rx) = mpsc::channel();
        let readers = [
            child.stdout.take().map(|out| read_lines(out, tx.clone())),
            child.stderr.take().map(|err| read_lines(err, tx.clone())),
        ];
        drop(tx);
        cancel.hold(child);

        let mut last = None;
        for line in rx.iter().filter_map(|raw: String| progress_line(&raw)) {
            last = Some(line.clone());
            sink(line);
        }
        for reader in readers.into_iter().flatten() {
            reader.join().ok();
        }
        let status = match cancel.take() {
            Some(mut child) => child.wait().map_err(|error| HostError(error.to_string()))?,
            None => return Err(HostError("Stopped.".into())),
        };
        if status.success() {
            Ok(())
        } else {
            Err(failure(last))
        }
    }
}

fn read_lines(
    pipe: impl Read + Send + 'static,
    tx: mpsc::Sender<String>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for line in BufReader::new(pipe).lines() {
            let Ok(line) = line else { break };
            if tx.send(line).is_err() {
                break;
            }
        }
    })
}

/// The standard output of a finished `limactl`, or its last error line.
fn finish(output: std::process::Output) -> Result<String, HostError> {
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(failure(
        stderr.lines().filter_map(progress_line).next_back(),
    ))
}

fn failure(last_line: Option<String>) -> HostError {
    let message = last_line.unwrap_or_else(|| "limactl failed.".into());
    let message = message
        .strip_prefix("fatal: ")
        .or_else(|| message.strip_prefix("error: "))
        .unwrap_or(&message);
    HostError(message.to_string())
}

fn spawn_error(binary: &Path, error: std::io::Error) -> HostError {
    HostError(format!("Cannot run {}: {error}", binary.display()))
}

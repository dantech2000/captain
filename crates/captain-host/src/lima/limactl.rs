//! Runs `limactl`, always with Captain's own `LIMA_HOME`, so it never sees or
//! changes the user's Lima VMs.

use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, mpsc};

use captain_core::HostError;

use super::args::progress_line;

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
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(failure(
            stderr.lines().filter_map(progress_line).next_back(),
        ))
    }

    /// Runs `args`, passes each progress line to `sink`, and blocks until it exits.
    /// The child waits in `running`, so another thread can kill it.
    pub fn stream(
        &self,
        args: &[String],
        running: &Mutex<Option<Child>>,
        sink: &mut dyn FnMut(String),
    ) -> Result<(), HostError> {
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
        *lock(running) = Some(child);

        let mut last = None;
        for line in rx.iter().filter_map(|raw: String| progress_line(&raw)) {
            last = Some(line.clone());
            sink(line);
        }
        for reader in readers.into_iter().flatten() {
            reader.join().ok();
        }
        let status = match lock(running).take() {
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

/// Kills the child waiting in `running`, if any.
pub fn kill(running: &Mutex<Option<Child>>) {
    if let Some(child) = lock(running).as_mut() {
        child.kill().ok();
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

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
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

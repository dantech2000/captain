//! Streams the output lines of a long command.

use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};

use captain_core::{EngineError, EngineStream};
use futures::StreamExt;
use futures::channel::mpsc::{self, UnboundedSender};

use crate::child::{Guarded, SharedChild};
use crate::compose::error_message;
use crate::process_group::own_group;

/// How many stderr lines Captain keeps to find the error of a failed command.
const TAIL_LINES: usize = 50;

/// Runs `command` with its output read on plain threads. The stream yields each
/// line of stdout and stderr, then ends, or ends with the error line of a failed
/// run. When the receiver drops the stream, the command is killed and reaped.
pub fn stream_lines(mut command: Command, name: &'static str) -> EngineStream<String> {
    let (tx, rx) = mpsc::unbounded();
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = match own_group(&mut command).spawn() {
        Ok(child) => child,
        Err(err) => {
            let error = EngineError::Api(format!("cannot run {name}: {err}"));
            tx.unbounded_send(Err(error)).ok();
            return rx.boxed();
        }
    };
    let stdout = child.stdout.take().map(|stdout| {
        let tx = tx.clone();
        std::thread::spawn(move || forward(stdout, &tx, &mut Vec::new()))
    });
    let stderr = child.stderr.take().map(|stderr| {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let mut tail = Vec::new();
            forward(stderr, &tx, &mut tail);
            tail
        })
    });
    let child = SharedChild::new(child);
    let waiter = child.clone();
    std::thread::spawn(move || {
        if let Some(thread) = stdout {
            thread.join().ok();
        }
        let tail = stderr
            .and_then(|thread| thread.join().ok())
            .unwrap_or_default();
        let result = match waiter.wait() {
            Ok(status) if status.success() => return,
            Ok(status) => error_message(&tail.join("\n"))
                .unwrap_or_else(|| format!("{name} exited with {status}")),
            Err(err) => format!("{name} stopped: {err}"),
        };
        tx.unbounded_send(Err(EngineError::Api(result))).ok();
    });
    Guarded::new(rx, child).boxed()
}

/// Sends each line of `pipe` and keeps the last ones in `tail`. Stops when the
/// receiver is gone.
fn forward(
    pipe: impl Read,
    tx: &UnboundedSender<Result<String, EngineError>>,
    tail: &mut Vec<String>,
) {
    for line in BufReader::new(pipe).lines() {
        let Ok(line) = line else {
            break;
        };
        if tail.len() == TAIL_LINES {
            tail.remove(0);
        }
        tail.push(line.clone());
        if tx.unbounded_send(Ok(line)).is_err() {
            return;
        }
    }
}

#[cfg(all(test, unix))]
mod tests;

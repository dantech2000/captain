//! Streams the output lines of a long command.

use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};

use captain_core::{EngineError, EngineStream};
use futures::StreamExt;
use futures::channel::mpsc::{self, UnboundedSender};

use crate::compose::error_message;

/// How many stderr lines Captain keeps to find the error of a failed command.
const TAIL_LINES: usize = 50;

/// Runs `command` on a plain thread. The stream yields each line of stdout and
/// stderr, then ends, or ends with the error line of a failed run. When the
/// receiver drops the stream, the command is killed.
pub fn stream_lines(mut command: Command, name: &'static str) -> EngineStream<String> {
    let (tx, rx) = mpsc::unbounded();
    std::thread::spawn(move || {
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(err) => {
                let error = EngineError::Api(format!("cannot run {name}: {err}"));
                tx.unbounded_send(Err(error)).ok();
                return;
            }
        };
        let stdout = child.stdout.take().map(|stdout| {
            let tx = tx.clone();
            std::thread::spawn(move || forward(stdout, &tx, &mut Vec::new()))
        });
        let mut tail = Vec::new();
        let mut open = child
            .stderr
            .take()
            .is_none_or(|stderr| forward(stderr, &tx, &mut tail));
        if !open {
            // Kill first, so the stdout thread sees the end of its pipe.
            child.kill().ok();
        }
        if let Some(thread) = stdout {
            open &= thread.join().unwrap_or(false);
        }
        if !open {
            child.kill().ok();
            child.wait().ok();
            return;
        }
        let result = match child.wait() {
            Ok(status) if status.success() => return,
            Ok(status) => error_message(&tail.join("\n"))
                .unwrap_or_else(|| format!("{name} exited with {status}")),
            Err(err) => format!("{name} stopped: {err}"),
        };
        tx.unbounded_send(Err(EngineError::Api(result))).ok();
    });
    rx.boxed()
}

/// Sends each line of `pipe` and keeps the last ones in `tail`. Returns false when
/// the receiver is gone.
fn forward(
    pipe: impl Read,
    tx: &UnboundedSender<Result<String, EngineError>>,
    tail: &mut Vec<String>,
) -> bool {
    for line in BufReader::new(pipe).lines() {
        let Ok(line) = line else {
            break;
        };
        if tail.len() == TAIL_LINES {
            tail.remove(0);
        }
        tail.push(line.clone());
        if tx.unbounded_send(Ok(line)).is_err() {
            return false;
        }
    }
    true
}

//! Runs the commands of `*.cli.exec` on plain threads: to the end, or line by line.
//! The process is killed and reaped when the page drops the stream.

use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::thread::JoinHandle;

use captain_core::extension::{BridgeEvent, BridgeStream, exec_result};
use futures::channel::mpsc::{self, UnboundedSender};
use futures::channel::oneshot;
use futures::{FutureExt, StreamExt};

use crate::child::{Guarded, SharedChild};
use crate::process::drain;
use crate::process_group::own_group;

/// Runs `command` to the end and answers with an `ExecResult`.
pub fn run(mut command: Command, cmd: String) -> BridgeStream {
    let (tx, rx) = oneshot::channel();
    let answer = rx
        .map(|event| event.unwrap_or_else(|_| BridgeEvent::error("the command stopped")))
        .into_stream();
    let mut child = match spawn(&mut command) {
        Ok(child) => child,
        Err(error) => {
            tx.send(BridgeEvent::error(format!("cannot run {cmd}: {error}")))
                .ok();
            return answer.boxed();
        }
    };
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let child = SharedChild::new(child);
    let waiter = child.clone();
    std::thread::spawn(move || {
        let stdout = stdout.join().unwrap_or_default();
        let stderr = stderr.join().unwrap_or_default();
        let event = match waiter.wait() {
            Ok(status) => exec_result(
                &cmd,
                status.code().unwrap_or(-1),
                String::from_utf8_lossy(&stdout).into_owned(),
                String::from_utf8_lossy(&stderr).into_owned(),
            ),
            Err(error) => BridgeEvent::error(format!("{cmd} stopped: {error}")),
        };
        tx.send(event).ok();
    });
    Guarded::new(answer, child).boxed()
}

/// Runs `command` and answers with each output line, then the exit code.
pub fn stream(mut command: Command, cmd: String) -> BridgeStream {
    let (tx, rx) = mpsc::unbounded();
    let mut child = match spawn(&mut command) {
        Ok(child) => child,
        Err(error) => {
            tx.unbounded_send(BridgeEvent::error(format!("cannot run {cmd}: {error}")))
                .ok();
            return rx.boxed();
        }
    };
    let readers = [
        child
            .stdout
            .take()
            .map(|pipe| forward(pipe, false, tx.clone())),
        child
            .stderr
            .take()
            .map(|pipe| forward(pipe, true, tx.clone())),
    ];
    let child = SharedChild::new(child);
    let waiter = child.clone();
    std::thread::spawn(move || {
        for reader in readers.into_iter().flatten() {
            reader.join().ok();
        }
        let code = waiter
            .wait()
            .ok()
            .and_then(|status| status.code())
            .unwrap_or(-1);
        tx.unbounded_send(BridgeEvent::Exit(code)).ok();
    });
    Guarded::new(rx, child).boxed()
}

fn spawn(command: &mut Command) -> std::io::Result<std::process::Child> {
    own_group(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

fn forward(
    pipe: impl Read + Send + 'static,
    stderr: bool,
    tx: UnboundedSender<BridgeEvent>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        for line in BufReader::new(pipe).lines() {
            let Ok(line) = line else {
                break;
            };
            if tx
                .unbounded_send(BridgeEvent::Output { stderr, line })
                .is_err()
            {
                break;
            }
        }
    })
}

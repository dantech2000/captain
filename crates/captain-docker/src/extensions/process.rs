//! Runs the commands of `*.cli.exec` on plain threads: to the end, or line by line
//! with the process killed when the page closes the stream.

use std::io::{BufRead, BufReader, Read};
use std::pin::Pin;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use captain_core::extension::{BridgeEvent, BridgeStream, exec_result};
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::channel::oneshot;
use futures::{FutureExt, Stream, StreamExt};

/// Runs `command` to the end and answers with an `ExecResult`.
pub fn run(mut command: Command, cmd: String) -> BridgeStream {
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || {
        command.stdin(Stdio::null());
        let event = match command.output() {
            Ok(output) => exec_result(
                &cmd,
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stdout).into_owned(),
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ),
            Err(error) => BridgeEvent::error(format!("cannot run {cmd}: {error}")),
        };
        tx.send(event).ok();
    });
    rx.map(|event| event.unwrap_or_else(|_| BridgeEvent::error("the command stopped")))
        .into_stream()
        .boxed()
}

/// Runs `command` and answers with each output line, then the exit code.
pub fn stream(mut command: Command, cmd: String) -> BridgeStream {
    let (tx, rx) = mpsc::unbounded();
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match command.spawn() {
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
    let child = Arc::new(Mutex::new(Some(child)));
    let waiter = child.clone();
    std::thread::spawn(move || {
        for reader in readers.into_iter().flatten() {
            reader.join().ok();
        }
        let child = waiter.lock().ok().and_then(|mut child| child.take());
        let code = child
            .and_then(|mut child| child.wait().ok())
            .and_then(|status| status.code())
            .unwrap_or(-1);
        tx.unbounded_send(BridgeEvent::Exit(code)).ok();
    });
    Running { rx, child }.boxed()
}

fn forward(
    pipe: impl Read + Send + 'static,
    stderr: bool,
    tx: UnboundedSender<BridgeEvent>,
) -> std::thread::JoinHandle<()> {
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

/// The events of a streaming command. Dropping it kills the command.
struct Running {
    rx: UnboundedReceiver<BridgeEvent>,
    child: Arc<Mutex<Option<Child>>>,
}

impl Stream for Running {
    type Item = BridgeEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx)
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        // The waiting thread takes the child after it exits; a running one is killed
        // here, which ends its pipes and lets that thread reap it.
        if let Ok(mut child) = self.child.lock()
            && let Some(child) = child.as_mut()
        {
            child.kill().ok();
        }
    }
}

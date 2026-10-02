//! Turns a PTY into an [`ExecSession`]. Three threads do the blocking work: one
//! reads the output, one writes the input, and one waits for the shell to exit.
//! Ending the session runs on a thread of its own ([`closing`]), so a write that
//! waits for room cannot hold it up.

use std::io::{ErrorKind, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use captain_core::EngineError;
use captain_core::model::{ExecInput, ExecResizer, ExecSession};
use captain_terminal::{Pty, PtyControl};
use futures::channel::{mpsc as channel, oneshot};
use futures::future::ready;
use futures::{FutureExt, StreamExt, stream};

use super::closing::Closing;
use super::output_budget::OutputBudget;

/// How long a hung-up shell gets to exit before it is killed.
const HANGUP_GRACE: Duration = Duration::from_secs(1);
/// How long the exit waits for the last output after the shell exited. Programs
/// the shell left running in the background can keep the output open for longer.
const OUTPUT_GRACE: Duration = Duration::from_millis(300);
/// The output bytes that may wait for the view. Past this the reader stops reading,
/// and a program that writes fast, such as `yes`, blocks in the PTY.
const QUEUED_OUTPUT: usize = 1024 * 1024;

/// A session for `pty` whose output starts with `banner`. Dropping the session's
/// input, as a closed tab does, ends the shell.
pub fn pty_session(pty: Pty, banner: String, command: Vec<String>) -> ExecSession {
    let Pty {
        reader,
        writer,
        control,
        child,
    } = pty;
    let (output_tx, output_rx) = channel::unbounded::<Vec<u8>>();
    let (input_tx, input_rx) = channel::unbounded::<Vec<u8>>();
    let (exit_tx, exit_rx) = oneshot::channel::<Option<i64>>();
    let (read_done_tx, read_done_rx) = mpsc::channel::<()>();
    let budget = OutputBudget::new(QUEUED_OUTPUT);
    let release = budget.release_handle();

    spawn("terminal-read", move || {
        read_loop(reader, &output_tx, &budget);
        read_done_tx.send(()).ok();
    });
    spawn("terminal-wait", move || {
        let code = child.wait();
        read_done_rx.recv_timeout(OUTPUT_GRACE).ok();
        exit_tx.send(code).ok();
    });
    let ending = control.clone();
    spawn("terminal-write", move || {
        let mut writer = writer;
        write_loop(&mut writer, input_rx);
        // The input went away: the tab closed or restarted. A shell that already
        // exited gets no signals.
        ending.terminate(HANGUP_GRACE);
    });

    let resize = control;
    ExecSession {
        command,
        input: ExecInput::new(input_tx),
        output: stream::once(ready(Ok(banner.into_bytes())))
            .chain(output_rx.map(move |bytes| {
                release.release(bytes.len());
                Ok(bytes)
            }))
            .boxed(),
        resizer: ExecResizer::new(move |cols, rows| {
            let result = resize
                .resize(cols, rows)
                .map_err(|error| EngineError::Api(error.to_string()));
            ready(result).boxed()
        }),
        exit: async move { Ok(exit_rx.await.ok().flatten()) }.boxed(),
    }
}

/// Ends the session of `control` on a thread of its own: a hangup, and a kill
/// after [`HANGUP_GRACE`] for what is left.
pub fn closing(control: PtyControl) -> Closing {
    Closing::spawn(move || control.terminate(HANGUP_GRACE))
}

fn spawn(name: &str, work: impl FnOnce() + Send + 'static) {
    if let Err(error) = thread::Builder::new().name(name.into()).spawn(work) {
        tracing::warn!(%error, "cannot start a terminal thread");
    }
}

/// Reads output until the shell's side closes, the session ends, or the view
/// stops reading.
fn read_loop(
    mut reader: Box<dyn Read + Send>,
    output: &channel::UnboundedSender<Vec<u8>>,
    budget: &OutputBudget,
) {
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if !budget.acquire(n) || output.unbounded_send(buf[..n].to_vec()).is_err() {
                    break;
                }
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            // macOS and Linux report EIO once the shell's side of the PTY closes.
            Err(_) => break,
        }
    }
}

/// Writes input until the session's input handle goes away or the session ends.
fn write_loop(writer: &mut Box<dyn Write + Send>, input: channel::UnboundedReceiver<Vec<u8>>) {
    for bytes in futures::executor::block_on_stream(input) {
        if writer
            .write_all(&bytes)
            .and_then(|()| writer.flush())
            .is_err()
        {
            break;
        }
    }
}

//! Turns a PTY into an [`ExecSession`]. Three threads do the blocking work: one
//! reads the output, one writes the input, and one waits for the shell to exit.

use std::io::{ErrorKind, Read, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use captain_core::EngineError;
use captain_core::model::{ExecInput, ExecResizer, ExecSession};
use captain_terminal::{Pty, PtyControl};
use futures::channel::{mpsc as channel, oneshot};
use futures::future::ready;
use futures::{FutureExt, StreamExt, stream};

/// How long a hung-up shell gets to exit before it is killed.
const HANGUP_GRACE: Duration = Duration::from_secs(1);
/// How long the exit waits for the last output after the shell exited. Programs
/// the shell left running in the background can keep the output open for longer.
const OUTPUT_GRACE: Duration = Duration::from_millis(300);

/// A session for `pty` whose output starts with `banner`. Dropping the session's
/// input, as a closed tab does, hangs up the shell.
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
    let exited = Arc::new(AtomicBool::new(false));

    spawn("terminal-read", move || {
        read_loop(reader, output_tx);
        read_done_tx.send(()).ok();
    });
    let flag = exited.clone();
    spawn("terminal-wait", move || {
        let code = child.wait();
        flag.store(true, Ordering::SeqCst);
        read_done_rx.recv_timeout(OUTPUT_GRACE).ok();
        exit_tx.send(code).ok();
    });
    let hangup = control.clone();
    spawn("terminal-write", move || {
        let mut writer = writer;
        write_loop(&mut writer, input_rx);
        // The tab closed or restarted while the shell runs. The writer goes only
        // after the shell, because dropping it types a newline and end of file.
        if !exited.load(Ordering::SeqCst) {
            end(&hangup, &exited);
        }
        drop(writer);
    });

    let resize = control.clone();
    ExecSession {
        command,
        input: ExecInput::new(input_tx),
        output: stream::once(ready(Ok(banner.into_bytes())))
            .chain(output_rx.map(Ok))
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

fn spawn(name: &str, work: impl FnOnce() + Send + 'static) {
    if let Err(error) = thread::Builder::new().name(name.into()).spawn(work) {
        tracing::warn!(%error, "cannot start a terminal thread");
    }
}

fn read_loop(mut reader: Box<dyn Read + Send>, output: channel::UnboundedSender<Vec<u8>>) {
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if output.unbounded_send(buf[..n].to_vec()).is_err() {
                    break;
                }
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            // macOS and Linux report EIO once the shell's side of the PTY closes.
            Err(_) => break,
        }
    }
}

/// Writes input until the session's input handle goes away.
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

/// Hangs up the shell and its foreground program, and kills them if they are still
/// there after [`HANGUP_GRACE`].
fn end(control: &PtyControl, exited: &AtomicBool) {
    control.hangup();
    let start = Instant::now();
    while start.elapsed() < HANGUP_GRACE {
        if exited.load(Ordering::SeqCst) {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    control.kill();
}

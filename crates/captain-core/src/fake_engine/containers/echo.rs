//! The fake exec: a session that writes its input straight back, like a TTY in echo
//! mode with nothing behind it.

use futures::channel::{mpsc, oneshot};
use futures::future::ready;
use futures::{FutureExt, StreamExt, stream};

use crate::model::{DEFAULT_SHELLS, ExecInput, ExecResizer, ExecSession, ExecSpec};
use crate::{EngineError, EngineFuture};

/// An echo session for `spec`. The output ends, and the exit resolves with code 0,
/// when every handle to the input is dropped.
pub fn echo_session(spec: ExecSpec) -> ExecSession {
    let command = if spec.uses_default_shell() {
        vec![DEFAULT_SHELLS[0].to_string()]
    } else {
        spec.cmd
    };
    let (input_tx, input_rx) = mpsc::unbounded::<Vec<u8>>();
    let (exit_tx, exit_rx) = oneshot::channel();
    let finished = stream::once(async move {
        exit_tx.send(Some(0)).ok();
    })
    .filter_map(|()| ready(None));
    let output = input_rx.map(Ok).chain(finished).boxed();
    let exit: EngineFuture<Option<i64>> = exit_rx
        .map(|code| code.map_err(|_| EngineError::Api("the fake exec was dropped".into())))
        .boxed();

    ExecSession {
        command,
        input: ExecInput::new(input_tx),
        output,
        resizer: ExecResizer::new(|_, _| ready(Ok(())).boxed()),
        exit,
    }
}

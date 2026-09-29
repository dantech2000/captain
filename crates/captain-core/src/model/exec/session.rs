use std::fmt;
use std::sync::Arc;

use futures::channel::mpsc::UnboundedSender;

use crate::{EngineError, EngineFuture, EngineStream};

/// A running `exec` with a TTY. Dropping it closes the connection, which ends the
/// command's input; an interactive shell exits on that.
pub struct ExecSession {
    /// The command that runs, for example `["/bin/bash"]`.
    pub command: Vec<String>,
    /// Bytes for the command's input.
    pub input: ExecInput,
    /// The command's output, as raw bytes. It ends when the command exits.
    pub output: EngineStream<Vec<u8>>,
    pub resizer: ExecResizer,
    /// Resolves after the output ends, with the exit code if the engine reports one.
    pub exit: EngineFuture<Option<i64>>,
}

impl ExecSession {
    /// The command as one line, for example `/bin/bash`.
    pub fn command_line(&self) -> String {
        self.command.join(" ")
    }
}

impl fmt::Debug for ExecSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecSession")
            .field("command", &self.command)
            .finish_non_exhaustive()
    }
}

/// The input side of an exec. Cloning it gives another handle to the same input.
#[derive(Debug, Clone)]
pub struct ExecInput {
    sender: UnboundedSender<Vec<u8>>,
}

impl ExecInput {
    pub fn new(sender: UnboundedSender<Vec<u8>>) -> Self {
        Self { sender }
    }

    /// Queues `bytes` for the command. Fails once the session has ended.
    pub fn send(&self, bytes: Vec<u8>) -> Result<(), EngineError> {
        if bytes.is_empty() {
            return Ok(());
        }
        self.sender
            .unbounded_send(bytes)
            .map_err(|_| EngineError::Api("the exec session has ended".into()))
    }
}

type ResizeFn = dyn Fn(u16, u16) -> EngineFuture<()> + Send + Sync;

/// Changes the size of an exec's TTY. Cloning it gives another handle.
#[derive(Clone)]
pub struct ExecResizer {
    resize: Arc<ResizeFn>,
}

impl ExecResizer {
    pub fn new(resize: impl Fn(u16, u16) -> EngineFuture<()> + Send + Sync + 'static) -> Self {
        Self {
            resize: Arc::new(resize),
        }
    }

    /// Tells the engine the TTY is now `cols` by `rows`.
    pub fn resize(&self, cols: u16, rows: u16) -> EngineFuture<()> {
        (self.resize)(cols, rows)
    }
}

impl fmt::Debug for ExecResizer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExecResizer")
    }
}

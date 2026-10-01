use std::sync::Arc;

use captain_core::model::{ExecSession, ExecSpec};
use captain_core::{Engine, EngineFuture};

use super::TerminalSource;

/// A shell in a container, through the engine's exec API. Dropping the session
/// closes the connection, so the shell reads end of file and exits.
pub struct ExecSource {
    pub engine: Arc<dyn Engine>,
    pub container: String,
}

impl TerminalSource for ExecSource {
    fn open(&self, cols: u16, rows: u16) -> EngineFuture<ExecSession> {
        self.engine
            .exec(&self.container, ExecSpec::shell(cols, rows))
    }
}

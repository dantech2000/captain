mod session;

pub use session::{ExecInput, ExecResizer, ExecSession};

use crate::model::EnvVar;

/// The shells the default command tries, in order.
pub const DEFAULT_SHELLS: [&str; 2] = ["/bin/bash", "/bin/sh"];

/// What `docker exec -it` needs to run a command in a running container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecSpec {
    /// The command and its arguments. Empty means the default shell: `/bin/bash` if
    /// the container has it, else `/bin/sh`.
    pub cmd: Vec<String>,
    /// Allocate a pseudo-terminal. Output is then one raw byte stream.
    pub tty: bool,
    pub cols: u16,
    pub rows: u16,
    /// Extra environment variables for the command.
    pub env: Vec<EnvVar>,
}

impl ExecSpec {
    /// An interactive default shell with a TTY of `cols` by `rows`. It sets `TERM` so
    /// programs send colors and cursor keys the emulator understands.
    pub fn shell(cols: u16, rows: u16) -> Self {
        Self {
            cmd: Vec::new(),
            tty: true,
            cols,
            rows,
            env: vec![
                EnvVar::parse("TERM=xterm-256color"),
                EnvVar::parse("COLORTERM=truecolor"),
            ],
        }
    }

    /// Whether the engine must pick the shell.
    pub fn uses_default_shell(&self) -> bool {
        self.cmd.is_empty()
    }

    /// The environment as `KEY=value` entries, the form the engine takes.
    pub fn env_entries(&self) -> Vec<String> {
        self.env
            .iter()
            .map(|var| format!("{}={}", var.key, var.value))
            .collect()
    }
}

#[cfg(test)]
mod tests;

//! The shell of Captain's terminal panel: the program it runs and the environment
//! it gets. See docs/features/0041-integrated-terminal.md.

mod env;
mod locate;
mod program;

pub use env::{ShellEnv, ShellEnvInput, shell_env};
pub use locate::local_env;
pub use program::{ShellProgram, shell_program};

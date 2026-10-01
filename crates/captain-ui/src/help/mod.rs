//! Hover help: every control declares a help sentence, and the status bar shows the
//! sentence of the control under the mouse. See feature 0029.

mod help_ext;
mod hover_batch;
mod hover_help;
mod liveness;

pub use help_ext::{CMD, HelpExt, cmd_key};
pub use hover_batch::hover_batch;
pub use hover_help::{Hint, clear, ensure, hover_help};

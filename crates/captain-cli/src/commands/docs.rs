//! `captain docs cli`: the Markdown reference of the commands.

use crate::cli::DocsCommand;
use crate::reference;

pub fn run(command: DocsCommand) {
    match command {
        DocsCommand::Cli => print!("{}", reference::markdown()),
    }
}

//! `captain completion <shell>`.

use clap::CommandFactory;
use clap_complete::Shell;

use crate::cli::Cli;

pub fn run(shell: Shell) {
    clap_complete::generate(
        shell,
        &mut Cli::command(),
        "captain",
        &mut std::io::stdout(),
    );
}

//! `captain`: control Captain Engine from the terminal, with or without the app.
//! See docs/features/0022-command-line.md.

mod cli;
mod commands;
mod context;
mod settings_keys;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let cli = cli::Cli::parse();
    match commands::run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("captain: {error:#}");
            ExitCode::FAILURE
        }
    }
}

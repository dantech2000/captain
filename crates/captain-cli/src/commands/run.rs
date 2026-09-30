//! Sends each command to its file.

use anyhow::Result;

use super::{
    completion, docker_env, info, kubernetes, list_settings, restart, set, shell, snapshot, start,
    status, stop, tools, version,
};
use crate::cli::{Cli, Command};
use crate::context::Context;

pub fn run(cli: Cli) -> Result<()> {
    let context = Context::new(cli.settings, cli.lima_home, cli.instance)?;
    match cli.command {
        Command::Start => start::run(&context),
        Command::Stop => stop::run(&context),
        Command::Restart => restart::run(&context),
        Command::Status { json } => status::run(&context, json),
        Command::Info { json } => info::run(&context, json),
        Command::ListSettings { json } => list_settings::run(&context, json),
        Command::Set { key, value } => set::run(&context, key, &value),
        Command::Shell { command } => shell::run(&context, &command),
        Command::Snapshot(command) => snapshot::run(&context, command),
        Command::Kubernetes(command) => kubernetes::run(&context, command),
        Command::Tools(command) => tools::run(&context, command),
        Command::DockerEnv => docker_env::run(&context),
        Command::Version => {
            version::run();
            Ok(())
        }
        Command::Completion { shell } => {
            completion::run(shell);
            Ok(())
        }
    }
}

//! The command-line arguments.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_complete::Shell;

use crate::settings_keys::SettingKey;

/// Control Captain Engine from the terminal.
#[derive(Debug, Parser)]
#[command(name = "captain", bin_name = "captain", version)]
pub struct Cli {
    /// The settings file to use instead of Captain's own.
    #[arg(long, global = true, env = "CAPTAIN_SETTINGS", value_name = "PATH")]
    pub settings: Option<PathBuf>,
    /// For tests: the `LIMA_HOME` of another Captain Engine instance. Its template,
    /// engine lock, and snapshots sit next to it.
    #[arg(
        long,
        global = true,
        hide = true,
        env = "CAPTAIN_LIMA_HOME",
        value_name = "PATH"
    )]
    pub lima_home: Option<PathBuf>,
    /// For tests: the Lima instance name, with `--lima-home`.
    #[arg(
        long,
        global = true,
        hide = true,
        env = "CAPTAIN_INSTANCE",
        value_name = "NAME",
        requires = "lima_home"
    )]
    pub instance: Option<String>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Start Captain Engine. The first start sets it up.
    Start,
    /// Stop Captain Engine.
    Stop,
    /// Stop and start Captain Engine, for example to apply new resources.
    Restart,
    /// Show the engine state, the Docker socket, and the versions.
    Status {
        /// Print JSON.
        #[arg(long)]
        json: bool,
    },
    /// Show the resources and the paths Captain Engine uses.
    Info {
        /// Print JSON.
        #[arg(long)]
        json: bool,
    },
    /// Show the settings that `captain set` can change.
    ListSettings {
        /// Print JSON.
        #[arg(long)]
        json: bool,
    },
    /// Change one setting. Quit Captain first.
    Set { key: SettingKey, value: String },
    /// Open a shell in Captain Engine's VM, or run one command in it.
    Shell {
        /// The command to run, after `--`.
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Save, list, restore, and delete snapshots of Captain Engine.
    #[command(subcommand)]
    Snapshot(SnapshotCommand),
    /// Turn Kubernetes (k3s in Captain Engine) on or off, show it, or reset it.
    #[command(subcommand)]
    Kubernetes(KubernetesCommand),
    /// Print the DOCKER_HOST for Captain Engine. Use: eval "$(captain docker-env)"
    DockerEnv,
    /// Print the version.
    Version,
    /// Print a shell completion script.
    Completion { shell: Shell },
}

/// `captain snapshot ...`. `NAME` is a snapshot's name or ID.
#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SnapshotCommand {
    /// Save the engine as a snapshot. The engine stops meanwhile.
    Create {
        /// The name. The default is the date and time.
        name: Option<String>,
        #[arg(long, short, default_value = "")]
        description: String,
        /// Do not ask before stopping the engine.
        #[arg(long, short)]
        yes: bool,
    },
    /// List the snapshots, newest first.
    List {
        /// Print JSON.
        #[arg(long)]
        json: bool,
    },
    /// Replace the engine with a snapshot. Quit Captain first.
    Restore {
        name: String,
        /// Do not ask first.
        #[arg(long, short)]
        yes: bool,
    },
    /// Delete a snapshot.
    Delete {
        name: String,
        /// Do not ask first.
        #[arg(long, short)]
        yes: bool,
    },
}

/// `captain kubernetes ...`. See docs/features/0024-kubernetes.md.
#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum KubernetesCommand {
    /// Turn Kubernetes on. A running engine starts it now. Quit Captain first.
    Enable {
        /// The k3s version, such as v1.36.4+k3s1. The default is the saved one, or
        /// the stable one.
        #[arg(long)]
        version: Option<String>,
        /// The port of the Kubernetes API on 127.0.0.1.
        #[arg(long)]
        port: Option<u16>,
        /// Install Traefik on ports 80 and 443.
        #[arg(long)]
        traefik: Option<bool>,
    },
    /// Turn Kubernetes off. The cluster keeps its state. Quit Captain first.
    Disable,
    /// Show the settings and the state of the cluster.
    Status {
        /// Print JSON.
        #[arg(long)]
        json: bool,
    },
    /// Delete the cluster's workloads and state, then start it again. Images stay.
    Reset {
        /// Do not ask first.
        #[arg(long, short)]
        yes: bool,
    },
}

#[cfg(test)]
mod tests;

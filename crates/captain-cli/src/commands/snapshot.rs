//! `captain snapshot create|list|restore|rename|delete`. The work is in `captain-host`, the
//! same code the Snapshots page uses. See docs/features/0023-snapshots.md.

mod create;
mod delete;
mod engine_cycle;
mod list;
mod rename;
mod restore;

use std::io::{BufRead, IsTerminal, Write};
use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use captain_core::EngineHost;
use captain_core::snapshot::{EngineSnapshots, Snapshot, find};
use futures::executor::block_on;

use crate::cli::SnapshotCommand;
use crate::context::Context;

pub fn run(context: &Context, command: SnapshotCommand) -> Result<()> {
    let host = context.host(&context.load_or_default());
    let snapshots = host
        .snapshots()
        .context("snapshots need Captain Engine, which runs on macOS only")?;
    match command {
        SnapshotCommand::Create {
            name,
            description,
            yes,
        } => create::run(&host, &snapshots, name, description, yes),
        SnapshotCommand::List { json } => list::run(&snapshots, json),
        SnapshotCommand::Restore { name, yes } => {
            restore::run(context, &host, &snapshots, &name, yes)
        }
        SnapshotCommand::Rename {
            name,
            new_name,
            description,
        } => rename::run(&snapshots, &name, new_name, description),
        SnapshotCommand::Delete { name, yes } => delete::run(&snapshots, &name, yes),
    }
}

/// The snapshot whose name or ID is `key`.
fn lookup(snapshots: &Arc<dyn EngineSnapshots>, key: &str) -> Result<Snapshot> {
    let list = block_on(snapshots.list())?;
    match find(&list.snapshots, key) {
        Some(snapshot) => Ok(snapshot.clone()),
        None => bail!("no snapshot is named {key:?}; see `captain snapshot list`"),
    }
}

/// Asks `question` on the terminal unless `yes`. Without a terminal it fails, so a
/// script must pass `--yes`.
pub(super) fn confirm(question: &str, yes: bool) -> Result<()> {
    if yes {
        return Ok(());
    }
    if !std::io::stdin().is_terminal() {
        bail!("{question} Add --yes to confirm.");
    }
    print!("{question} [y/N] ");
    std::io::stdout().flush()?;
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer)?;
    match answer.trim() {
        "y" | "Y" | "yes" => Ok(()),
        _ => bail!("cancelled"),
    }
}

/// True if the engine runs now.
fn running(host: &Arc<dyn EngineHost>) -> Result<bool> {
    Ok(block_on(host.status())?.is_running())
}

//! `captain snapshot create [NAME] [--description TEXT] [--yes]`.

use std::sync::Arc;

use anyhow::Result;
use captain_core::EngineHost;
use captain_core::format::bytes_label;
use captain_core::snapshot::EngineSnapshots;
use futures::executor::block_on;

use super::engine_cycle::with_engine_stopped;
use super::{confirm, running};

pub fn run(
    host: &Arc<dyn EngineHost>,
    snapshots: &Arc<dyn EngineSnapshots>,
    name: Option<String>,
    description: String,
    yes: bool,
) -> Result<()> {
    let name = name.unwrap_or_else(default_name);
    let was_running = running(host)?;
    if was_running {
        confirm("Captain Engine will stop and start again. Continue?", yes)?;
    }
    let snapshot = with_engine_stopped(host, was_running, || {
        println!("Saving the snapshot {name:?}.");
        Ok(block_on(snapshots.create(name.clone(), description))?)
    })?;
    println!(
        "Saved {:?} ({}).",
        snapshot.metadata.name,
        bytes_label(snapshot.metadata.disk_allocated)
    );
    Ok(())
}

/// The date and time, for example `2026-09-29 14:05`, as on the Snapshots page.
fn default_name() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()
}

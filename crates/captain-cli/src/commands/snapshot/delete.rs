//! `captain snapshot delete NAME [--yes]`. The engine keeps running.

use std::sync::Arc;

use anyhow::Result;
use captain_core::snapshot::EngineSnapshots;
use futures::executor::block_on;

use super::{confirm, lookup};

pub fn run(snapshots: &Arc<dyn EngineSnapshots>, key: &str, yes: bool) -> Result<()> {
    let snapshot = lookup(snapshots, key)?;
    let name = &snapshot.metadata.name;
    confirm(&format!("Delete the snapshot {name:?}?"), yes)?;
    block_on(snapshots.delete(snapshot.id.clone()))?;
    println!("Deleted {name:?}.");
    Ok(())
}

//! `captain snapshot rename NAME NEW_NAME [--description TEXT]`. The engine keeps
//! running.

use std::sync::Arc;

use anyhow::Result;
use captain_core::snapshot::EngineSnapshots;
use futures::executor::block_on;

use super::lookup;

pub fn run(
    snapshots: &Arc<dyn EngineSnapshots>,
    key: &str,
    new_name: String,
    description: Option<String>,
) -> Result<()> {
    let snapshot = lookup(snapshots, key)?;
    let description = description.unwrap_or(snapshot.metadata.description);
    let edited = block_on(snapshots.edit(snapshot.id, new_name, description))?;
    println!("Saved {:?}.", edited.metadata.name);
    Ok(())
}

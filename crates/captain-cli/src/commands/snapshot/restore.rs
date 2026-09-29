//! `captain snapshot restore NAME [--yes]`. It refuses while the app runs, because
//! the app owns the settings file and its own view of the engine.

use std::sync::Arc;

use anyhow::{Result, bail};
use captain_core::EngineHost;
use captain_core::snapshot::EngineSnapshots;
use futures::executor::block_on;

use super::engine_cycle::with_engine_stopped;
use super::{confirm, lookup, running};
use crate::context::Context;

pub fn run(
    context: &Context,
    host: &Arc<dyn EngineHost>,
    snapshots: &Arc<dyn EngineSnapshots>,
    key: &str,
    yes: bool,
) -> Result<()> {
    if context.app_running() {
        bail!("Captain is running. Restore from the Snapshots page, or quit Captain first.");
    }
    let snapshot = lookup(snapshots, key)?;
    let name = &snapshot.metadata.name;
    let was_running = running(host)?;
    let restart = if was_running {
        " Captain Engine will stop and start again."
    } else {
        ""
    };
    confirm(
        &format!("The current engine state is replaced by {name:?}.{restart} Continue?"),
        yes,
    )?;
    with_engine_stopped(host, was_running, || {
        println!("Restoring {name:?}.");
        let restored = block_on(snapshots.restore(snapshot.id.clone()))?;
        // Keep the settings in line with the restored `lima.yaml`, so the next start
        // does not edit it back.
        let resources = restored.metadata.resources;
        let mut settings = context.load_or_default();
        settings.engine_resources = Some(resources);
        settings.save(&context.settings_path)?;
        block_on(host.set_resources(resources))?;
        Ok(())
    })?;
    println!("Restored {name:?}.");
    Ok(())
}

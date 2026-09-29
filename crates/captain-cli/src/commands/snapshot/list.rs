//! `captain snapshot list [--json]`.

use std::sync::Arc;

use anyhow::Result;
use captain_core::format::bytes_label;
use captain_core::snapshot::EngineSnapshots;
use chrono::TimeZone;
use futures::executor::block_on;
use serde::Serialize;

#[derive(Serialize)]
struct Row<'a> {
    id: &'a str,
    name: &'a str,
    description: &'a str,
    created: u64,
    disk_allocated: u64,
}

pub fn run(snapshots: &Arc<dyn EngineSnapshots>, json: bool) -> Result<()> {
    let list = block_on(snapshots.list())?;
    if json {
        let rows: Vec<Row> = list
            .snapshots
            .iter()
            .map(|snapshot| Row {
                id: &snapshot.id,
                name: &snapshot.metadata.name,
                description: &snapshot.metadata.description,
                created: snapshot.metadata.created,
                disk_allocated: snapshot.metadata.disk_allocated,
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }
    if list.snapshots.is_empty() {
        println!("No snapshots. Save one with `captain snapshot create`.");
        return Ok(());
    }
    for snapshot in &list.snapshots {
        let metadata = &snapshot.metadata;
        println!(
            "{:<24} {:<16} {:>8}  {}",
            metadata.name,
            date(metadata.created),
            bytes_label(metadata.disk_allocated),
            metadata.description
        );
    }
    Ok(())
}

/// Unix seconds as local `YYYY-MM-DD HH:MM`.
fn date(seconds: u64) -> String {
    chrono::Local
        .timestamp_opt(seconds as i64, 0)
        .single()
        .map_or_else(String::new, |time| {
            time.format("%Y-%m-%d %H:%M").to_string()
        })
}

//! `captain set <key> <value>`. The app keeps the settings in memory and writes the
//! whole file on each change, so `set` refuses while the app runs. It holds the
//! settings lock from the read to the write.

use anyhow::{Result, anyhow};
use captain_core::HostStatus;
use futures::executor::block_on;

use crate::context::Context;
use crate::settings_keys::{SettingKey, check_disk_floor};

pub fn run(context: &Context, key: SettingKey, value: &str) -> Result<()> {
    let current_disk = match key {
        SettingKey::Disk => current_disk(context)?,
        _ => None,
    };
    let saved = context.update_settings(
        "Captain is running. Change this in Settings, or quit Captain first.",
        |settings| {
            key.apply(settings, value, &context.machine)
                .and_then(|()| check_disk_floor(settings, &context.machine, current_disk))
                .map_err(|why| anyhow!(why))?;
            Ok(key.value(settings, &context.machine))
        },
    )?;
    println!("{} is {saved}.", key.name());
    if key.is_resource() && engine_running(context) {
        println!("Run `captain restart` to apply it.");
    }
    Ok(())
}

/// The size of Captain Engine's disk now, or `None` before the first setup or for
/// a host without a machine. A status check reads it.
fn current_disk(context: &Context) -> Result<Option<u64>> {
    let host = context.host(&context.load_or_default());
    if !host.can_control() {
        return Ok(None);
    }
    let status = block_on(host.status());
    disk_floor(status.ok(), host.current_disk()).map_err(|why| anyhow!(why))
}

/// The disk size that a new one cannot go below. Only a status check that finds no
/// machine allows any size; when the status or the disk of an existing machine
/// cannot be read, `set disk` refuses, since a disk cannot shrink.
fn disk_floor(status: Option<HostStatus>, current: Option<u64>) -> Result<Option<u64>, String> {
    match (current, status) {
        (Some(bytes), _) => Ok(Some(bytes)),
        (None, Some(HostStatus::NotCreated)) => Ok(None),
        (None, status) => {
            let why = status
                .as_ref()
                .and_then(HostStatus::detail)
                .map(|why| format!(" ({why})"))
                .unwrap_or_default();
            Err(format!(
                "Captain cannot read the size of Captain Engine's disk{why}, so it cannot check the new size. Try again when `captain status` works."
            ))
        }
    }
}

fn engine_running(context: &Context) -> bool {
    let host = context.host(&context.load_or_default());
    block_on(host.status()).is_ok_and(|status| status.is_running())
}

#[cfg(test)]
mod tests;

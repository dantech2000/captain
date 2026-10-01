//! `captain set <key> <value>`. The app keeps the settings in memory and writes the
//! whole file on each change, so `set` refuses while the app runs. It holds the
//! settings lock from the read to the write.

use anyhow::{Result, anyhow};
use futures::executor::block_on;

use crate::context::Context;
use crate::settings_keys::{SettingKey, check_disk_floor};

pub fn run(context: &Context, key: SettingKey, value: &str) -> Result<()> {
    let current_disk = (key == SettingKey::Disk)
        .then(|| current_disk(context))
        .flatten();
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

/// The size of Captain Engine's disk now, or `None` before the first setup. A
/// status check reads it.
fn current_disk(context: &Context) -> Option<u64> {
    let host = context.host(&context.load_or_default());
    block_on(host.status()).ok()?;
    host.current_disk()
}

fn engine_running(context: &Context) -> bool {
    let host = context.host(&context.load_or_default());
    block_on(host.status()).is_ok_and(|status| status.is_running())
}

#[cfg(test)]
mod tests;

//! `captain set <key> <value>`. The app keeps the settings in memory and writes the
//! whole file on each change, so `set` refuses while the app runs.

use anyhow::{Result, anyhow, bail};
use futures::executor::block_on;

use crate::context::Context;
use crate::settings_keys::SettingKey;

pub fn run(context: &Context, key: SettingKey, value: &str) -> Result<()> {
    if context.app_running() {
        bail!("Captain is running. Change this in Settings, or quit Captain first.");
    }
    let mut settings = context.load()?;
    key.apply(&mut settings, value, &context.machine)
        .map_err(|why| anyhow!(why))?;
    settings.save(&context.settings_path)?;
    let saved = key.value(&settings, &context.machine);
    println!("{} is {saved}.", key.name());
    if key.is_resource() && engine_running(context) {
        println!("Run `captain restart` to apply it.");
    }
    Ok(())
}

fn engine_running(context: &Context) -> bool {
    let host = context.host(&context.load_or_default());
    block_on(host.status()).is_ok_and(|status| status.is_running())
}

#[cfg(test)]
mod tests;

//! A LaunchAgent in `~/Library/LaunchAgents`. `SMAppService` needs a signed bundle,
//! and Captain.app is unsigned for now. launchd loads the agent at the next login.

use std::io;
use std::path::PathBuf;

use captain_core::behavior::login_item::{LOGIN_ITEM_ID, launch_agent_plist};

use super::file;

fn path() -> io::Result<PathBuf> {
    file::in_dir(
        dirs::home_dir(),
        "Library/LaunchAgents",
        &format!("{LOGIN_ITEM_ID}.plist"),
    )
}

/// True if the LaunchAgent exists.
pub fn is_enabled() -> io::Result<bool> {
    Ok(path()?.exists())
}

pub fn set(enabled: bool) -> io::Result<()> {
    let path = path()?;
    if enabled {
        file::write(&path, &launch_agent_plist(&file::program()?))
    } else {
        file::remove(&path)
    }
}

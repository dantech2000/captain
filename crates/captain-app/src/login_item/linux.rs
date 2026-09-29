//! An XDG autostart entry in `$XDG_CONFIG_HOME/autostart`.

use std::path::PathBuf;
use std::{fs, io};

use captain_core::behavior::login_item::{LOGIN_ITEM_ID, autostart_enabled, autostart_entry};

use super::file;

fn path() -> io::Result<PathBuf> {
    file::in_dir(
        dirs::config_dir(),
        "autostart",
        &format!("{LOGIN_ITEM_ID}.desktop"),
    )
}

/// True if the entry exists and the desktop has not turned it off.
pub fn is_enabled() -> io::Result<bool> {
    match fs::read_to_string(path()?) {
        Ok(entry) => Ok(autostart_enabled(&entry)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

pub fn set(enabled: bool) -> io::Result<()> {
    let path = path()?;
    if enabled {
        file::write(&path, &autostart_entry(&file::program()?))
    } else {
        file::remove(&path)
    }
}

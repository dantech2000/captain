//! A value under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.

use std::{env, io};

use captain_core::behavior::login_item::{RUN_KEY, RUN_VALUE_NAME, run_command};
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};

/// True if the `Run` key has Captain's value.
pub fn is_enabled() -> io::Result<bool> {
    let key = match RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN_KEY, KEY_READ) {
        Ok(key) => key,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    match key.get_raw_value(RUN_VALUE_NAME) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

pub fn set(enabled: bool) -> io::Result<()> {
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(RUN_KEY)?;
    if enabled {
        let program = env::current_exe()?;
        key.set_value(RUN_VALUE_NAME, &run_command(&program.to_string_lossy()))
    } else {
        match key.delete_value(RUN_VALUE_NAME) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            result => result,
        }
    }
}

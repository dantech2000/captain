use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::edit::FileEdit;
use super::{FileProblem, Settings, jsonc, overrides, validate};
use crate::file_replace::{Mode, backup_once, replace, sibling};
use crate::link_target::link_target;

/// Why the settings file could not be read or written.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("cannot access {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    /// The file is not valid JSON. Captain never writes over it.
    #[error("{problem}")]
    Parse { path: PathBuf, problem: FileProblem },
}

/// The settings in a file, and the values in it that this build cannot use. Each of
/// those keys has its default in `settings`.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedSettings {
    pub settings: Settings,
    pub problems: Vec<FileProblem>,
    /// The file's text, so a watcher can tell a change. Empty for a missing file.
    pub text: String,
}

impl LoadedSettings {
    /// Reads the text of a settings file. Only bad JSON is an error.
    pub fn parse(text: String) -> Result<Self, FileProblem> {
        let (settings, problems) = validate::read(&text)?;
        Ok(Self {
            settings,
            problems,
            text,
        })
    }

    /// True if the file sets `key`, such as `theme` or `kubernetes.port`, rather
    /// than leaving it at its default.
    pub fn sets(&self, key: &str) -> bool {
        let path: Vec<String> = key.split('.').map(String::from).collect();
        jsonc::parse(&self.text).is_ok_and(|file| overrides::get(&file, &path).is_some())
    }
}

impl Settings {
    /// Reads the settings at `path`. A missing file gives the defaults. See
    /// [`Settings::read`] for the values it cannot use.
    pub fn load(path: &Path) -> Result<Self, SettingsError> {
        Self::read(path).map(|loaded| loaded.settings)
    }

    /// Reads the settings at `path` with their problems. Only bad JSON is an error.
    pub fn read(path: &Path) -> Result<LoadedSettings, SettingsError> {
        let text = read_text(path)?.unwrap_or_default();
        LoadedSettings::parse(text).map_err(|problem| SettingsError::Parse {
            path: path.to_path_buf(),
            problem,
        })
    }

    /// Writes the keys that differ between `before` and `after` into the file at
    /// `path`, in place: comments, key order, and other keys stay. A value equal to
    /// its default removes its key. A missing file is created with a `$schema`
    /// line. A file from before format 2 is migrated first. A file with bad JSON
    /// is an error and stays as it is.
    ///
    /// A writer holds [`settings_lock_path`](crate::process_lock::settings_lock_path)
    /// meanwhile. A symlinked file stays a link: the write goes to its target.
    pub fn save_change(
        path: &Path,
        before: &Settings,
        after: &Settings,
    ) -> Result<(), SettingsError> {
        edit_file(path, true, |edit| {
            edit.apply(after, &overrides::changed(before, after));
        })
    }

    /// Writes these settings into the file at `path`: the keys that differ from
    /// what the file holds now. See [`Settings::save_change`].
    pub fn save(&self, path: &Path) -> Result<(), SettingsError> {
        let on_file = Self::load(path)?;
        Self::save_change(path, &on_file, self)
    }

    /// Drops the keys equal to their defaults from a file written before format 2,
    /// once. The old file stays in `settings.json.captain-backup`. A missing file
    /// stays missing.
    pub fn migrate_file(path: &Path) -> Result<(), SettingsError> {
        edit_file(path, false, |_| {})
    }

    /// Like [`Settings::migrate_file`], but a missing file is created: a starter
    /// file with a note, the `$schema` line, and examples in comments. Captain
    /// calls this at each start, so the file is there to open and edit.
    pub fn prepare_file(path: &Path) -> Result<(), SettingsError> {
        edit_file(path, true, |_| {})
    }
}

/// Opens the file, migrates it if it is old, runs `change`, and writes it back if
/// its text changed.
fn edit_file(
    path: &Path,
    create: bool,
    change: impl FnOnce(&mut FileEdit),
) -> Result<(), SettingsError> {
    let target = &link_target(path);
    let text = match read_text(target)? {
        Some(text) => text,
        None if create => String::new(),
        None => return Ok(()),
    };
    let mut edit = FileEdit::open(&text).map_err(|problem| SettingsError::Parse {
        path: path.to_path_buf(),
        problem,
    })?;
    if edit.needs_migration() {
        backup(path, target)?;
        edit.migrate();
    }
    change(&mut edit);
    let new = edit.text();
    if new == text {
        return Ok(());
    }
    write(target, &new)
}

fn read_text(path: &Path) -> Result<Option<String>, SettingsError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(io_error(path, source)),
    }
}

/// Copies the file to `settings.json.captain-backup` next to `link`, unless a
/// backup is already there.
fn backup(link: &Path, target: &Path) -> Result<(), SettingsError> {
    let backup = sibling(link, "captain-backup");
    backup_once(target, &backup).map_err(|source| io_error(&backup, source))
}

/// Writes a temporary file next to `path` and renames it, so a crash never leaves
/// half a file.
fn write(path: &Path, text: &str) -> Result<(), SettingsError> {
    replace(path, text.as_bytes(), Mode::Keep).map_err(|source| io_error(path, source))
}

fn io_error(path: &Path, source: io::Error) -> SettingsError {
    SettingsError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests;

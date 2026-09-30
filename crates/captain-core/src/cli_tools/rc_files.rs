//! The shell files that put `~/.captain/bin` on `PATH`: which ones, whether Captain
//! may write them, and the block in them.

use std::path::{Path, PathBuf};

use super::rc_block::{END, START, with_block, without_block};
use crate::file_replace::{Mode, backup_once, replace, sibling};
use crate::link_target::link_target;

/// Home-manager and Nix link files into the read-only store.
const NIX_STORE: &str = "/nix/store";

/// A shell whose file Captain may change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
}

impl Shell {
    pub fn name(self) -> &'static str {
        match self {
            Shell::Zsh => "zsh",
            Shell::Bash => "bash",
            Shell::Fish => "fish",
        }
    }

    /// The line that puts `~/.captain/bin` first on `PATH`.
    pub fn path_line(self) -> &'static str {
        match self {
            Shell::Zsh | Shell::Bash => r#"export PATH="$HOME/.captain/bin:$PATH""#,
            Shell::Fish => "fish_add_path --global --move --path $HOME/.captain/bin",
        }
    }

    /// Captain's marked block with [`Self::path_line`].
    pub fn block(self) -> String {
        format!(
            "{START}\n# Added by Captain. Set command_line_tools.path to manual in Captain's settings file to remove it.\n{}\n{END}\n",
            self.path_line()
        )
    }
}

/// A shell file for one shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RcFile {
    pub shell: Shell,
    pub path: PathBuf,
}

/// The files for the shells on this computer: a shell counts when its file (or
/// fish's config folder) exists, or when it is the login shell.
pub fn rc_files(home: &Path, login_shell: Option<&Path>) -> Vec<RcFile> {
    let login = login_shell.and_then(Path::file_name);
    let is_login = |shell: Shell| login.is_some_and(|name| name == shell.name());
    let exists = |path: &Path| std::fs::symlink_metadata(path).is_ok();
    let mut files = Vec::new();
    let zshrc = home.join(".zshrc");
    if exists(&zshrc) || is_login(Shell::Zsh) {
        files.push((Shell::Zsh, zshrc));
    }
    let bash = [".bash_profile", ".bashrc"]
        .map(|name| home.join(name))
        .into_iter()
        .find(|path| exists(path));
    match bash {
        Some(path) => files.push((Shell::Bash, path)),
        None if is_login(Shell::Bash) => files.push((Shell::Bash, home.join(".bash_profile"))),
        None => {}
    }
    let fish = home.join(".config/fish");
    if exists(&fish) || is_login(Shell::Fish) {
        files.push((Shell::Fish, fish.join("conf.d/captain.fish")));
    }
    files
        .into_iter()
        .map(|(shell, path)| RcFile { shell, path })
        .collect()
}

/// Whether Captain may write a shell file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RcAccess {
    Writable,
    /// Why not. The user adds the line.
    Skip(String),
}

impl RcAccess {
    /// True if home-manager or Nix owns the file: it links into, or sits in,
    /// `/nix/store`. The user then adds the folder in their Nix config.
    pub fn in_nix_store(&self) -> bool {
        matches!(self, RcAccess::Skip(why) if why.contains(NIX_STORE))
    }
}

/// Captain writes only a regular file it can write, or a new file in a folder it
/// can write. It skips links (home-manager's links into `/nix/store`, for
/// example) and files that `managed` says a dotfile manager owns.
pub fn rc_access(path: &Path, managed: impl Fn(&Path) -> bool) -> RcAccess {
    rc_access_in(path, Path::new(NIX_STORE), managed)
}

fn rc_access_in(path: &Path, nix_store: &Path, managed: impl Fn(&Path) -> bool) -> RcAccess {
    let skip = |why: String| RcAccess::Skip(why);
    let in_store = |path: &Path| path.starts_with(nix_store);
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            let target = link_target(path);
            match in_store(&target) {
                true => skip("It links into /nix/store (home-manager or Nix).".into()),
                false => skip(format!("It links to {}.", target.display())),
            }
        }
        Ok(meta) if !meta.is_file() => skip("It is not a regular file.".into()),
        Ok(_) if managed(path) => skip("chezmoi manages it.".into()),
        Ok(_) => match std::fs::OpenOptions::new().append(true).open(path) {
            Ok(_) => RcAccess::Writable,
            Err(_) => skip("Captain cannot write it.".into()),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let Some(dir) = path.ancestors().skip(1).find(|dir| dir.exists()) else {
                return skip("Its folder does not exist.".into());
            };
            let dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
            let store = nix_store
                .canonicalize()
                .unwrap_or_else(|_| nix_store.into());
            let read_only = std::fs::metadata(&dir).is_ok_and(|meta| meta.permissions().readonly());
            if dir.starts_with(&store) {
                skip("Its folder is in /nix/store (home-manager or Nix).".into())
            } else if read_only {
                skip("Captain cannot create it.".into())
            } else if managed(path) {
                skip("chezmoi manages it.".into())
            } else {
                RcAccess::Writable
            }
        }
        Err(error) => skip(format!("Captain cannot read it: {error}")),
    }
}

/// What a shell file has now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RcState {
    /// Captain's block.
    Added,
    /// A line of the user's own that names `.captain/bin`.
    Present,
    Missing,
}

/// Reads the file, also through a link.
pub fn rc_state(path: &Path) -> RcState {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    if text.lines().any(|line| line.trim() == START) {
        RcState::Added
    } else if text.contains(".captain/bin") {
        RcState::Present
    } else {
        RcState::Missing
    }
}

/// Adds or refreshes Captain's block. Check [`rc_access`] first.
pub fn add_block(rc: &RcFile) -> Result<(), String> {
    let text = match std::fs::read_to_string(&rc.path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(write_error(rc, error)),
    };
    write(rc, &with_block(&text, &rc.shell.block()))
}

/// Removes Captain's block. Captain's own fish file goes when nothing else is in
/// it. Check [`rc_access`] first. Returns true if the file changed.
pub fn remove_block(rc: &RcFile) -> Result<bool, String> {
    let Ok(text) = std::fs::read_to_string(&rc.path) else {
        return Ok(false);
    };
    let Some(rest) = without_block(&text) else {
        return Ok(false);
    };
    match rc.shell == Shell::Fish && rest.trim().is_empty() {
        true => std::fs::remove_file(&rc.path).map_err(|error| write_error(rc, error))?,
        false => write(rc, &rest)?,
    }
    Ok(true)
}

/// Replaces the file through a synced temporary file, with the same permissions.
/// Before Captain's first change, the old file goes to `<name>.captain-backup`.
/// Captain's own fish file has no backup.
fn write(rc: &RcFile, text: &str) -> Result<(), String> {
    let fail = |error: std::io::Error| write_error(rc, error);
    if rc.shell != Shell::Fish {
        backup_once(&rc.path, &sibling(&rc.path, "captain-backup")).map_err(fail)?;
    }
    replace(&rc.path, text.as_bytes(), Mode::Keep).map_err(fail)
}

fn write_error(rc: &RcFile, error: std::io::Error) -> String {
    format!("Cannot write {}: {error}", rc.path.display())
}

#[cfg(all(test, unix))]
mod tests;

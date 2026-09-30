//! Shows a step before it runs, and runs it: a command through the user's login
//! shell, so the client's command and what it needs (such as `node`) are on PATH
//! as in a terminal, or an edit that replaces the file safely.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use super::paths::ClientPaths;
use super::plan::ClientStep;
use crate::cli_tools::command::output_within;
use crate::file_replace::{Mode, backup_once, commit, sibling, temp_path, write_new};

/// How long a client's installer may take.
const TIMEOUT: Duration = Duration::from_secs(60);

impl ClientStep {
    /// What the sheet shows before it runs: the command line, the link, or the
    /// changed lines of the file.
    pub fn preview(&self, paths: &ClientPaths) -> String {
        match self {
            ClientStep::Run(argv) => shell_line(argv),
            ClientStep::Open(link) => link.clone(),
            ClientStep::Edit {
                path,
                before,
                after,
            } => format!("{}\n{}", paths.tilde(path), line_diff(before, after)),
        }
    }
}

/// `argv` as one line a shell reads back the same, with single quotes where needed.
pub fn shell_line(argv: &[String]) -> String {
    argv.iter()
        .map(|arg| {
            let plain = !arg.is_empty()
                && arg
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_./:=@+".contains(c));
            if plain {
                arg.clone()
            } else {
                format!("'{}'", arg.replace('\'', r"'\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `argv` run by `shell` as a login shell, as a new terminal would run it.
pub fn login_shell_command(shell: &Path, argv: &[String]) -> Command {
    let mut command = Command::new(shell);
    command.arg("-lic").arg(shell_line(argv));
    command
}

/// Runs a [`ClientStep::Run`] with `launch`, or writes a [`ClientStep::Edit`]. The
/// caller opens a [`ClientStep::Open`] link. An edit refuses a file that changed
/// since the step read it, and keeps a copy of the first version it replaces.
pub fn run_step(step: &ClientStep, launch: &dyn Fn(&[String]) -> Command) -> Result<(), String> {
    match step {
        ClientStep::Run(argv) => {
            let output = output_within(launch(argv), TIMEOUT)
                .map_err(|error| format!("{} did not run: {error}", shell_line(argv)))?;
            if output.status.success() {
                return Ok(());
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let why = [stderr.trim(), stdout.trim()]
                .into_iter()
                .find(|text| !text.is_empty())
                .unwrap_or("no output");
            Err(format!("{} failed: {why}", argv[0]))
        }
        ClientStep::Open(_) => Ok(()),
        ClientStep::Edit {
            path,
            before,
            after,
        } => edit_file(path, before, after),
    }
}

/// Writes `after` over `path` if the file still holds `before`. The new file is
/// staged and the backup made first, so the check comes right before the rename.
/// A file Captain creates gets an empty backup, which tells Remove that Captain
/// added everything in it.
fn edit_file(path: &Path, before: &str, after: &str) -> Result<(), String> {
    let cannot =
        |error: std::io::Error| format!("Captain cannot write {}: {error}", path.display());
    let temp = temp_path(path);
    write_new(&temp, path, after.as_bytes(), Mode::Keep).map_err(cannot)?;
    let backup = sibling(path, "captain-backup");
    let checked = backup_once(path, &backup)
        .map_err(cannot)
        .and_then(|()| unchanged(path, before));
    if let Err(why) = checked {
        std::fs::remove_file(&temp).ok();
        return Err(why);
    }
    commit(&temp, path).map_err(cannot)?;
    if !backup.exists() {
        std::fs::File::create_new(&backup).ok();
    }
    Ok(())
}

/// Ok if the file at `path` still holds `before`. A missing file holds nothing.
fn unchanged(path: &Path, before: &str) -> Result<(), String> {
    let now = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("Captain cannot read {}: {error}", path.display())),
    };
    if now != before {
        return Err(format!(
            "{} changed since Captain read it. Try again.",
            path.display()
        ));
    }
    Ok(())
}

/// The lines that differ between `before` and `after`, with `-` and `+`, after the
/// lines they share at the start and the end.
pub fn line_diff(before: &str, after: &str) -> String {
    let old: Vec<&str> = before.lines().collect();
    let new: Vec<&str> = after.lines().collect();
    let start = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let end = old[start..]
        .iter()
        .rev()
        .zip(new[start..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let removed = old[start..old.len() - end]
        .iter()
        .map(|line| format!("- {line}"));
    let added = new[start..new.len() - end]
        .iter()
        .map(|line| format!("+ {line}"));
    removed.chain(added).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests;

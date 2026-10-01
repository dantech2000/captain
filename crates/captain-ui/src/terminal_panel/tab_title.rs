use std::path::Path;

/// A tab's title: the title the shell set, else the folder's name, or `~` for the
/// home folder.
pub fn tab_title(shell_title: Option<String>, cwd: &Path, home: Option<&Path>) -> String {
    if let Some(title) = shell_title.filter(|title| !title.trim().is_empty()) {
        return title;
    }
    if home == Some(cwd) {
        return "~".into();
    }
    cwd.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| cwd.display().to_string())
}

#[cfg(test)]
mod tests;

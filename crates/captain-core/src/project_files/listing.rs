use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use crate::model::ComposeProject;

/// What an [`EditableFile`] holds, which picks its checks and highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Compose,
    Dockerfile,
}

/// A file of a Compose project that the editor may open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditableFile {
    /// The absolute path, as the project names it. It may be a symlink.
    pub path: PathBuf,
    pub kind: FileKind,
    /// For a Dockerfile, the services that build with it, sorted.
    pub services: Vec<String>,
    /// For a Dockerfile, the build context folder.
    pub context: Option<PathBuf>,
}

impl EditableFile {
    /// The path relative to `dir`, for the file list, for example `app/Dockerfile`.
    pub fn label(&self, dir: &Path) -> String {
        self.path
            .strip_prefix(dir)
            .unwrap_or(&self.path)
            .display()
            .to_string()
    }
}

/// The Compose files of `project` (its `config_files` label) that lie inside its
/// working folder, in the label's order. Without a working folder there is none.
pub fn compose_files(project: &ComposeProject) -> Vec<EditableFile> {
    let Some(dir) = project.working_dir.as_deref().map(Path::new) else {
        return Vec::new();
    };
    project
        .config_files
        .iter()
        .map(|file| dir.join(file))
        .filter(|path| is_inside(dir, path))
        .map(|path| EditableFile {
            path,
            kind: FileKind::Compose,
            services: Vec::new(),
            context: None,
        })
        .collect()
}

/// The Dockerfile of each service that builds from a local folder, read from the
/// output of `docker compose config --format json`. Services that share one file
/// share its entry. Files outside `dir`, remote contexts, and `dockerfile_inline`
/// are left out.
pub fn dockerfiles(config: &str, dir: &Path) -> Result<Vec<EditableFile>, String> {
    let config: Value = serde_json::from_str(config)
        .map_err(|error| format!("cannot read the Compose config: {error}"))?;
    let mut files: BTreeMap<PathBuf, EditableFile> = BTreeMap::new();
    let services = config.get("services").and_then(Value::as_object);
    for (service, entry) in services.into_iter().flatten() {
        let Some(build) = entry.get("build") else {
            continue;
        };
        let Some(context) = build.get("context").and_then(Value::as_str) else {
            continue;
        };
        if context.contains("://") || context.starts_with("git@") {
            continue;
        }
        if build.get("dockerfile_inline").is_some() {
            continue;
        }
        let context = dir.join(context);
        let name = build
            .get("dockerfile")
            .and_then(Value::as_str)
            .unwrap_or("Dockerfile");
        let path = context.join(name);
        if !is_inside(dir, &path) {
            continue;
        }
        let file = files.entry(path.clone()).or_insert_with(|| EditableFile {
            path,
            kind: FileKind::Dockerfile,
            services: Vec::new(),
            context: Some(context),
        });
        file.services.push(service.clone());
    }
    let mut files: Vec<EditableFile> = files.into_values().collect();
    for file in &mut files {
        file.services.sort();
    }
    Ok(files)
}

/// True if `path` lies in `dir` or below it, judged on the text of the paths with
/// `.` and `..` resolved. A symlink inside the folder counts as inside, wherever
/// it points; saving writes to its target.
pub fn is_inside(dir: &Path, path: &Path) -> bool {
    let (dir, path) = (normalize(dir), normalize(path));
    path != dir && path.starts_with(&dir)
}

fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests;

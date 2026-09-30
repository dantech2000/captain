use std::path::{Path, PathBuf};

use super::disk::{TextVersion, disk_version};
use crate::model::ComposeProject;

/// The versions of the files a preview of `up` read: the Compose files, the
/// project's `.env`, and the Dockerfiles the editor knows. Apply compares them
/// with the disk, so `up` never runs a change the preview did not show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputVersions(Vec<(PathBuf, Option<TextVersion>)>);

impl InputVersions {
    /// Reads the versions now. A missing file counts as `None`, so a file that
    /// appears or goes away is a change too.
    pub fn read(project: &ComposeProject, dockerfiles: &[PathBuf]) -> Self {
        let dir = Path::new(project.working_dir.as_deref().unwrap_or_default());
        let paths = project
            .config_files
            .iter()
            .map(|file| dir.join(file))
            .chain(project.working_dir.is_some().then(|| dir.join(".env")))
            .chain(dockerfiles.iter().cloned());
        Self(
            paths
                .map(|path| {
                    let version = disk_version(&path);
                    (path, version)
                })
                .collect(),
        )
    }

    /// True if a file is not at the version read any more.
    pub fn changed(&self) -> bool {
        self.0
            .iter()
            .any(|(path, version)| disk_version(path) != *version)
    }
}

#[cfg(test)]
mod tests;

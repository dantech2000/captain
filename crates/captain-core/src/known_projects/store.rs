use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::file_replace::{Mode, replace};
use crate::model::ComposeProject;

/// The file format this build writes.
const FORMAT: u32 = 1;

/// `~/.captain/projects.json` under `home`.
pub fn known_projects_path(home: &Path) -> PathBuf {
    home.join(".captain").join("projects.json")
}

/// One project Captain created or opened.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnownProject {
    /// The Compose project name, as `docker compose config` gives it.
    pub name: String,
    /// The project folder. Compose runs there.
    pub dir: PathBuf,
    /// The Compose files, relative to `dir` unless absolute.
    pub files: Vec<String>,
    /// When Captain recorded it, in Unix seconds.
    pub added: i64,
}

impl KnownProject {
    /// The project as the Compose runner and the Project page take it: no
    /// services, the folder, and the files as full paths.
    pub fn compose_project(&self) -> ComposeProject {
        ComposeProject {
            name: self.name.clone(),
            working_dir: Some(self.dir.display().to_string()),
            config_files: self
                .files
                .iter()
                .map(|file| self.dir.join(file).display().to_string())
                .collect(),
            services: Vec::new(),
        }
    }
}

/// Why the list could not be read, written, or changed.
#[derive(Debug, thiserror::Error)]
pub enum KnownProjectsError {
    #[error("cannot access {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    /// Captain never writes over a file it cannot read.
    #[error("{path} is not a project list Captain can read: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    /// Compose names are unique per engine, so Captain keeps one folder per name.
    #[error("Captain already knows a project named {name}, in {}", dir.display())]
    NameTaken { name: String, dir: PathBuf },
}

#[derive(Serialize, Deserialize)]
struct FileFormat {
    version: u32,
    #[serde(default)]
    projects: Vec<KnownProject>,
}

/// The known projects, in the order they were added.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KnownProjects {
    pub projects: Vec<KnownProject>,
}

impl KnownProjects {
    /// Reads the list at `path`. A missing file is an empty list.
    pub fn load(path: &Path) -> Result<Self, KnownProjectsError> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => {
                return Err(KnownProjectsError::Io {
                    path: path.to_path_buf(),
                    source,
                });
            }
        };
        let file: FileFormat =
            serde_json::from_str(&text).map_err(|source| KnownProjectsError::Parse {
                path: path.to_path_buf(),
                source,
            })?;
        Ok(Self {
            projects: file.projects,
        })
    }

    /// Writes the list to `path` in one atomic replace, and creates its folder.
    pub fn save(&self, path: &Path) -> Result<(), KnownProjectsError> {
        let file = FileFormat {
            version: FORMAT,
            projects: self.projects.clone(),
        };
        let mut text = serde_json::to_string_pretty(&file).expect("the list serializes");
        text.push('\n');
        replace(path, text.as_bytes(), Mode::Keep).map_err(|source| KnownProjectsError::Io {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Records `project`. The same name in the same folder takes the new files and
    /// keeps its place; the same name in another folder is refused.
    pub fn add(&mut self, project: KnownProject) -> Result<(), KnownProjectsError> {
        match self.projects.iter_mut().find(|p| p.name == project.name) {
            Some(known) if known.dir == project.dir => known.files = project.files,
            Some(known) => {
                return Err(KnownProjectsError::NameTaken {
                    name: known.name.clone(),
                    dir: known.dir.clone(),
                });
            }
            None => self.projects.push(project),
        }
        Ok(())
    }

    /// Forgets the project `name`. True if it was known.
    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.projects.len();
        self.projects.retain(|p| p.name != name);
        self.projects.len() != before
    }
}

#[cfg(test)]
mod tests;

//! The Compose projects Captain created or opened, kept in
//! `~/.captain/projects.json`, so a project stays in the sidebar after `down`.
//! See docs/features/0040-new-projects.md.

mod folder;
mod merge;
mod store;

pub use folder::{compose_files_in, project_name_from_config};
pub use merge::{known_match, stopped_known};
pub use store::{KnownProject, KnownProjects, KnownProjectsError, known_projects_path};

use std::path::Path;

use super::KnownProject;
use crate::model::ComposeProject;

/// The known project that `project`, found from container labels, is: the same
/// name and the same working folder.
pub fn known_match<'a>(
    project: &ComposeProject,
    known: &'a [KnownProject],
) -> Option<&'a KnownProject> {
    let dir = project.working_dir.as_deref().map(Path::new)?;
    known
        .iter()
        .find(|k| k.name == project.name && k.dir == dir)
}

/// The known projects that no project in `live` covers, as stopped projects with
/// no services, sorted by name. A live project with the same name covers a known
/// one even from another folder, because Compose names are unique per engine.
pub fn stopped_known(live: &[ComposeProject], known: &[KnownProject]) -> Vec<ComposeProject> {
    let mut stopped: Vec<ComposeProject> = known
        .iter()
        .filter(|k| live.iter().all(|p| p.name != k.name))
        .map(KnownProject::compose_project)
        .collect();
    stopped.sort_by(|a, b| a.name.cmp(&b.name));
    stopped
}

#[cfg(test)]
mod tests;

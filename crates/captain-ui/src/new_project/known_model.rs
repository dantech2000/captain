use std::path::PathBuf;

use captain_core::known_projects::{
    KnownProject, KnownProjects, KnownProjectsError, known_projects_path,
};
use gpui_kit::*;

/// The projects Captain created or opened, from `~/.captain/projects.json`.
pub struct KnownModel {
    path: Option<PathBuf>,
    list: KnownProjects,
    /// Why the file could not be read. Captain then never writes it.
    broken: Option<String>,
}

struct Shared(Entity<KnownModel>);

impl Global for Shared {}

/// The one model the sidebar, the Project page, and the New sheet share. The first
/// call reads the file.
pub fn known_model(cx: &mut App) -> Entity<KnownModel> {
    if let Some(shared) = cx.try_global::<Shared>() {
        return shared.0.clone();
    }
    let path = std::env::home_dir().map(|home| known_projects_path(&home));
    let model = cx.new(|_| KnownModel::read(path));
    cx.set_global(Shared(model.clone()));
    model
}

/// The known projects, without creating the model. Empty before
/// [`known_model`] first runs.
pub fn known_projects(cx: &App) -> Vec<KnownProject> {
    cx.try_global::<Shared>()
        .map(|shared| shared.0.read(cx).projects().to_vec())
        .unwrap_or_default()
}

impl KnownModel {
    fn read(path: Option<PathBuf>) -> Self {
        let (list, broken) = match path.as_deref().map(KnownProjects::load) {
            Some(Ok(list)) => (list, None),
            Some(Err(error)) => {
                tracing::warn!(%error, "cannot read the known projects");
                (KnownProjects::default(), Some(error.to_string()))
            }
            None => (KnownProjects::default(), None),
        };
        Self { path, list, broken }
    }

    pub fn projects(&self) -> &[KnownProject] {
        &self.list.projects
    }

    /// Records `project` and saves the file.
    pub fn add(&mut self, project: KnownProject, cx: &mut Context<Self>) -> Result<(), String> {
        let mut list = self.list.clone();
        list.add(project).map_err(|error| error.to_string())?;
        self.save(list, cx)
    }

    /// Forgets the project `name` and saves the file. Files stay on disk.
    pub fn remove(&mut self, name: &str, cx: &mut Context<Self>) -> Result<(), String> {
        let mut list = self.list.clone();
        if !list.remove(name) {
            return Ok(());
        }
        self.save(list, cx)
    }

    fn save(&mut self, list: KnownProjects, cx: &mut Context<Self>) -> Result<(), String> {
        if let Some(broken) = &self.broken {
            return Err(format!("{broken}. Fix or remove the file first."));
        }
        let Some(path) = &self.path else {
            return Err("Captain cannot find your home folder.".into());
        };
        list.save(path)
            .map_err(|error: KnownProjectsError| error.to_string())?;
        self.list = list;
        cx.notify();
        Ok(())
    }
}

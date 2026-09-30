use std::collections::HashMap;
use std::path::PathBuf;

use captain_core::project_files::{EditableFile, compose_files};
use gpui_kit::*;

use super::FileEditor;
use crate::project::ProjectView;
use crate::project::view_tabs::ProjectTab;

/// The Dockerfiles of the shown project, read from `docker compose config`.
#[derive(Default)]
pub enum DockerfileList {
    /// Not read yet, or no `docker compose`.
    #[default]
    None,
    Loading,
    Ready(Vec<EditableFile>),
    Failed(String),
}

/// The Files tab's state. Open files keep their unsaved edits when the page shows
/// another entry or tab.
#[derive(Default)]
pub struct FilesState {
    pub dockerfiles: DockerfileList,
    /// The project the Dockerfile list belongs to.
    listed: Option<String>,
    load: Option<Task<()>>,
    /// Open files by path.
    pub editors: HashMap<PathBuf, Entity<FileEditor>>,
    /// The file shown for each project.
    shown: HashMap<String, PathBuf>,
    /// The preview or `up` that runs for a file.
    pub(super) apply: Option<Task<()>>,
}

impl FilesState {
    /// Forgets the Dockerfile list, so the next render reads it again, for
    /// example after a saved Compose file changed a build context.
    pub fn forget_list(&mut self) {
        self.dockerfiles = DockerfileList::None;
        self.listed = None;
        self.load = None;
    }
}

impl ProjectView {
    /// The Compose files, then the Dockerfiles, of the shown project.
    pub(crate) fn editable_files(&self) -> Vec<EditableFile> {
        let Some(project) = &self.project else {
            return Vec::new();
        };
        let mut files = compose_files(project);
        if let DockerfileList::Ready(list) = &self.files.dockerfiles {
            files.extend(list.iter().cloned());
        }
        files
    }

    /// The open file the tab shows for the shown project.
    pub(crate) fn shown_editor(&self) -> Option<Entity<FileEditor>> {
        let project = self.project.as_ref()?;
        let path = self.files.shown.get(&project.name)?;
        self.files.editors.get(path).cloned()
    }

    /// Reads the Dockerfile list and opens the first file, when the Files tab
    /// shows a project for the first time.
    pub(crate) fn prepare_files(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.tab != ProjectTab::Files {
            return;
        }
        self.load_dockerfiles(cx);
        if self.shown_editor().is_none()
            && let Some(file) = self.editable_files().into_iter().next()
        {
            self.open_file(file, window, cx);
        }
    }

    fn load_dockerfiles(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.project.clone() else {
            return;
        };
        if self.files.listed.as_ref() == Some(&project.name) {
            return;
        }
        self.files.listed = Some(project.name.clone());
        let Some(runner) = self.workspace.read(cx).project_runner() else {
            self.files.dockerfiles = DockerfileList::None;
            return;
        };
        self.files.dockerfiles = DockerfileList::Loading;
        let read = runner.dockerfiles(&project);
        self.files.load = Some(cx.spawn(async move |this, cx| {
            let result = read.await;
            this.update(cx, |this, cx| {
                this.files.dockerfiles = match result {
                    Ok(files) => DockerfileList::Ready(files),
                    Err(error) => DockerfileList::Failed(error.to_string()),
                };
                cx.notify();
            })
            .ok();
        }));
    }

    /// Shows `file`, and opens it first if it is not open.
    pub(crate) fn open_file(
        &mut self,
        file: EditableFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(project) = self.project.clone() else {
            return;
        };
        let path = file.path.clone();
        if !self.files.editors.contains_key(&path) {
            let runner = self.workspace.read(cx).project_runner();
            let view = cx.entity().downgrade();
            let project = project.clone();
            let editor = cx.new(|cx| FileEditor::new(file, project, runner, view, window, cx));
            self.files.editors.insert(path.clone(), editor);
        }
        self.files.shown.insert(project.name, path);
        cx.notify();
    }
}

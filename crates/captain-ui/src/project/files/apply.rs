//! Save and apply: save the file, preview `up` with a dry run, confirm, run `up`.
//! Rebuild: save a Dockerfile, then `up -d --build <service>`.

use std::path::PathBuf;
use std::sync::Arc;

use captain_core::ProjectRunner;
use captain_core::model::ComposeProject;
use captain_core::project_files::InputVersions;
use gpui_kit::*;

use super::files_state::DockerfileList;
use super::{FileEditor, preview_dialog};
use crate::project::{ProjectNotice, ProjectView};

impl ProjectView {
    /// Saves `editor`'s file, then shows what `up` would change.
    pub(crate) fn save_and_apply(
        &mut self,
        editor: Entity<FileEditor>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((project, runner)) = self.apply_target(&editor, cx) {
            self.preview_apply(project, runner, editor, false, window, cx);
        }
    }

    /// Runs the dry run and asks. `again` is true when the files changed while the
    /// last preview was open.
    fn preview_apply(
        &mut self,
        project: ComposeProject,
        runner: Arc<dyn ProjectRunner>,
        editor: Entity<FileEditor>,
        again: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let inputs = InputVersions::read(&project, &self.dockerfile_paths());
        // A saved Compose file may point a service at another Dockerfile.
        self.files.forget_list();
        set_busy(&editor, Some("Checking what up would change..."), cx);
        let preview = runner.preview_up(&project);
        self.files.apply = Some(cx.spawn_in(window, async move |this, cx| {
            let result = preview.await;
            this.update_in(cx, |this, window, cx| {
                set_busy(&editor, None, cx);
                match result {
                    Ok(preview) => {
                        let view = cx.entity();
                        let asked = preview_dialog::Asked {
                            preview,
                            project,
                            inputs,
                            again,
                        };
                        preview_dialog::open(asked, view, editor, window, cx);
                    }
                    Err(error) => this.action_failed(
                        "Cannot preview the change",
                        &error.to_string(),
                        &editor,
                        cx,
                    ),
                }
            })
            .ok();
        }));
    }

    /// Runs `up` after the user confirmed a preview, if the files are still the
    /// ones it read. Otherwise the preview runs again, so `up` never makes a change
    /// the user did not see.
    pub(super) fn confirm_apply(
        &mut self,
        project: ComposeProject,
        inputs: InputVersions,
        editor: Entity<FileEditor>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(runner) = self.workspace.read(cx).project_runner() else {
            return;
        };
        if !inputs.changed() {
            self.apply_files(project, Vec::new(), editor, cx);
        } else {
            self.preview_apply(project, runner, editor, true, window, cx);
        }
    }

    /// Saves `editor`'s Dockerfile, then builds `service` again and recreates it.
    pub(crate) fn save_and_rebuild(
        &mut self,
        editor: Entity<FileEditor>,
        service: String,
        cx: &mut Context<Self>,
    ) {
        if let Some((project, _)) = self.apply_target(&editor, cx) {
            self.apply_files(project, vec![service], editor, cx);
        }
    }

    /// Runs `up` for `project`, or `up --build` for `build`. Compose's output goes
    /// to the project log.
    pub(super) fn apply_files(
        &mut self,
        project: ComposeProject,
        build: Vec<String>,
        editor: Entity<FileEditor>,
        cx: &mut Context<Self>,
    ) {
        let Some(runner) = self.workspace.read(cx).project_runner() else {
            return;
        };
        let doing = match build.first() {
            Some(service) => format!("Rebuilding {service}..."),
            None => "Applying...".into(),
        };
        set_busy(&editor, Some(&doing), cx);
        let run = runner.apply_up(&project, &build);
        self.files.apply = Some(cx.spawn(async move |this, cx| {
            let result = run.await;
            this.update(cx, |this, cx| {
                set_busy(&editor, None, cx);
                match result {
                    Ok(output) => {
                        this.log.update(cx, |log, cx| log.push_output(&output, cx));
                        cx.emit(ProjectNotice::Applied {
                            project: project.name,
                            rebuilt: build.first().cloned(),
                        });
                    }
                    Err(error) => {
                        let error = error.to_string();
                        this.log.update(cx, |log, cx| log.push_output(&error, cx));
                        let title = format!("Compose could not apply {}", project.name);
                        this.action_failed(&title, &error, &editor, cx);
                    }
                }
            })
            .ok();
        }));
    }

    /// Shows why an editor action failed under the editor, where it stays until the
    /// next action, and in a toast that hides itself, so it never covers the
    /// editor's buttons.
    fn action_failed(
        &mut self,
        title: &str,
        error: &str,
        editor: &Entity<FileEditor>,
        cx: &mut Context<Self>,
    ) {
        let why = error
            .lines()
            .map(str::trim)
            .rfind(|line| !line.is_empty())
            .unwrap_or_default();
        let line = format!("{title}: {why}");
        editor.update(cx, |editor, cx| {
            editor.action_error = Some(line.into());
            cx.notify();
        });
        cx.emit(ProjectNotice::EditorFailed {
            title: title.to_string(),
        });
    }

    /// The Dockerfiles in the list, for the versions a preview reads.
    fn dockerfile_paths(&self) -> Vec<PathBuf> {
        match &self.files.dockerfiles {
            DockerfileList::Ready(files) => files.iter().map(|file| file.path.clone()).collect(),
            _ => Vec::new(),
        }
    }

    /// Saves the file and returns the project and runner to apply it with. `None`
    /// when the save failed; the editor then says why.
    fn apply_target(
        &mut self,
        editor: &Entity<FileEditor>,
        cx: &mut Context<Self>,
    ) -> Option<(ComposeProject, Arc<dyn ProjectRunner>)> {
        let runner = self.workspace.read(cx).project_runner()?;
        let project = self.project.clone()?;
        let dirty = editor.read(cx).dirty;
        if dirty && !editor.update(cx, |editor, cx| editor.save(cx)) {
            return None;
        }
        Some((project, runner))
    }
}

/// Sets what the editor waits for. A new action clears the last one's error.
fn set_busy(editor: &Entity<FileEditor>, busy: Option<&str>, cx: &mut App) {
    editor.update(cx, |editor, cx| {
        if busy.is_some() {
            editor.action_error = None;
        }
        editor.busy = busy.map(|text| SharedString::from(text.to_string()));
        cx.notify();
    });
}

//! Save and apply: save the file, preview `up` with a dry run, confirm, run `up`.
//! Rebuild: save a Dockerfile, then `up -d --build <service>`.

use captain_core::model::ComposeProject;
use gpui_kit::*;

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
        let Some((project, runner)) = self.apply_target(&editor, cx) else {
            return;
        };
        // A saved Compose file may point a service at another Dockerfile.
        self.files.forget_list();
        set_busy(&editor, Some("Checking what up would change..."), cx);
        let preview = runner.preview_up(&project);
        self.files.apply = Some(cx.spawn_in(window, async move |this, cx| {
            let result = preview.await;
            this.update_in(cx, |_, window, cx| {
                set_busy(&editor, None, cx);
                match result {
                    Ok(preview) => {
                        let view = cx.entity();
                        preview_dialog::open(preview, project, view, editor, window, cx);
                    }
                    Err(error) => cx.emit(ProjectNotice::Failed {
                        title: "Cannot preview the change".into(),
                        error: error.to_string(),
                    }),
                }
            })
            .ok();
        }));
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
                        cx.emit(ProjectNotice::Failed {
                            title: format!("Compose could not apply {}", project.name),
                            error,
                        });
                    }
                }
            })
            .ok();
        }));
    }

    /// Saves the file and returns the project and runner to apply it with. `None`
    /// when the save failed; the editor then says why.
    fn apply_target(
        &mut self,
        editor: &Entity<FileEditor>,
        cx: &mut Context<Self>,
    ) -> Option<(
        ComposeProject,
        std::sync::Arc<dyn captain_core::ProjectRunner>,
    )> {
        let runner = self.workspace.read(cx).project_runner()?;
        let project = self.project.clone()?;
        let dirty = editor.read(cx).dirty;
        if dirty && !editor.update(cx, |editor, cx| editor.save(cx)) {
            return None;
        }
        Some((project, runner))
    }
}

fn set_busy(editor: &Entity<FileEditor>, busy: Option<&str>, cx: &mut App) {
    editor.update(cx, |editor, cx| {
        editor.busy = busy.map(|text| SharedString::from(text.to_string()));
        cx.notify();
    });
}

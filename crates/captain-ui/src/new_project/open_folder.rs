use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use captain_core::EngineError;
use captain_core::known_projects::{KnownProject, compose_files_in};
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::finish::open_project;
use super::known_model;
use super::new_sheet::{NewSheet, SheetStatus};

impl NewSheet {
    /// Asks for a folder or a Compose file with the system dialog.
    pub(super) fn pick_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: true,
            multiple: false,
            prompt: Some("Open".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            this.update_in(cx, |this, window, cx| this.check(path, window, cx))
                .ok();
        })
        .detach();
    }

    /// Finds the Compose files of `path` and checks them with `docker compose
    /// config`, which also gives the project name.
    fn check(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let (dir, files) = match path.parent() {
            Some(parent) if path.is_file() => (parent.to_path_buf(), vec![path.clone()]),
            _ => (path.clone(), compose_files_in(&path)),
        };
        let Some(first) = files.first() else {
            self.fail(
                format!(
                    "{} has no compose.yaml, compose.yml, docker-compose.yml, or docker-compose.yaml.",
                    path.display()
                ),
                cx,
            );
            return;
        };
        let Some(runner) = self.host.workspace.read(cx).project_runner() else {
            self.fail(
                "Captain needs docker compose to open a project. Diagnostics shows why it is missing.".into(),
                cx,
            );
            return;
        };
        let name = first.file_name().unwrap_or_default().to_string_lossy();
        self.status = SheetStatus::Checking(name.into_owned());
        cx.notify();
        let read = runner.project_name(&dir, &files);
        self.check = Some(cx.spawn_in(window, async move |this, cx| {
            let result = read.await;
            this.update_in(cx, |this, window, cx| {
                this.record(result, dir, files, window, cx)
            })
            .ok();
        }));
    }

    /// Remembers the checked project, closes the sheet, and shows the project's
    /// files.
    fn record(
        &mut self,
        result: Result<String, EngineError>,
        dir: PathBuf,
        files: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = match result {
            Ok(name) => name,
            Err(error) => return self.fail(error.to_string(), cx),
        };
        let project = KnownProject {
            name: name.clone(),
            files: files.iter().map(|file| relative(file, &dir)).collect(),
            dir,
            added: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |since| since.as_secs() as i64),
        };
        let added = known_model(cx).update(cx, |model, cx| model.add(project, cx));
        if let Err(error) = added {
            return self.fail(error, cx);
        }
        window.close_dialog(cx);
        open_project(&self.host, name, cx);
    }

    fn fail(&mut self, error: String, cx: &mut Context<Self>) {
        self.status = SheetStatus::Failed(error);
        cx.notify();
    }
}

/// `file` relative to `dir` when it is inside it, else the full path.
fn relative(file: &Path, dir: &Path) -> String {
    file.strip_prefix(dir).unwrap_or(file).display().to_string()
}

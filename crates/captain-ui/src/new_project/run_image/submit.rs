use captain_core::model::RunSpec;
use captain_core::new_project::{ComposeDoc, NewFile, image_project_name, project_files};
use futures::StreamExt;
use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::choice::pull_error;
use super::form_state::RunImage;
use crate::images::Started;
use crate::new_project::finish::create_project;
use crate::new_project::name_check::name_problem;

impl RunImage {
    /// The project name and its files, `compose.yaml` first, or the first
    /// problem. Secret values go in `.env`.
    pub(super) fn compose(&self, cx: &App) -> Result<(String, Vec<NewFile>), String> {
        let form = self.form(cx);
        let name = form.name.clone();
        if let Some(problem) = name_problem(&name, &self.host, cx) {
            return Err(problem);
        }
        let service = form.to_service().map_err(|error| error.to_string())?;
        let doc = ComposeDoc {
            name: Some(name.clone()),
            services: vec![(image_project_name(&self.repository), service)],
        };
        Ok((name, project_files(doc)?))
    }

    /// What the main button does now, or why it cannot.
    pub(super) fn check(&self, cx: &App) -> Result<(), String> {
        if self.save_as_project {
            return self.compose(cx).map(drop);
        }
        self.form(cx)
            .to_spec()
            .map(drop)
            .map_err(|error| error.to_string())
    }

    /// Saves the project, or runs the container.
    pub(super) fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.picking || self.busy.is_some() {
            return;
        }
        if self.save_as_project {
            let result = self
                .compose(cx)
                .and_then(|(name, files)| create_project(&self.host, &name, &files, window, cx));
            self.error = result.err();
            cx.notify();
            return;
        }
        match self.form(cx).to_spec() {
            Ok(spec) => self.run(spec, window, cx),
            Err(error) => {
                self.error = Some(error.to_string());
                cx.notify();
            }
        }
    }

    /// Pulls the image when the engine does not have it, then creates and starts
    /// the container. The sheet closes when the engine accepts it.
    fn run(&mut self, spec: RunSpec, window: &mut Window, cx: &mut Context<Self>) {
        let Some(engine) = self.host.workspace.read(cx).engine() else {
            self.error = Some("Captain is not connected to an engine.".into());
            cx.notify();
            return;
        };
        let generation = self
            .images
            .as_ref()
            .map(|images| images.read(cx).generation());
        let missing = !self.local.contains(&spec.image) && !spec.image.starts_with("sha256:");
        self.busy = Some(match missing {
            true => format!("Pulling {}\u{2026}", spec.image),
            false => "Starting\u{2026}".into(),
        });
        self.error = None;
        let images = self.images.clone();
        self.task = Some(cx.spawn_in(window, async move |this, cx| {
            if missing {
                let mut messages = engine.pull_image(&spec.image);
                while let Some(message) = messages.next().await {
                    if let Err(error) = message {
                        let error = pull_error(&spec.image, &error.to_string());
                        this.update(cx, |this, cx| this.failed(error, cx)).ok();
                        return;
                    }
                }
            }
            let name = spec.name.clone();
            let result = engine.run_container(spec).await;
            this.update_in(cx, |this, window, cx| match result {
                Ok(id) => {
                    let name = name.unwrap_or_else(|| id[..id.len().min(12)].to_string());
                    match (images, generation) {
                        (Some(images), Some(generation)) => {
                            let started = Started { id, name };
                            images
                                .update(cx, |state, cx| state.set_started(started, generation, cx));
                        }
                        _ => window.push_notification(
                            Notification::success(format!("Started the container {name}.")),
                            cx,
                        ),
                    }
                    this.busy = None;
                    window.close_dialog(cx);
                }
                Err(error) => {
                    tracing::warn!(%error, "running a container failed");
                    this.failed(format!("Run failed: {error}"), cx);
                }
            })
            .ok();
        }));
        cx.notify();
    }

    fn failed(&mut self, error: String, cx: &mut Context<Self>) {
        self.busy = None;
        self.error = Some(error);
        cx.notify();
    }
}

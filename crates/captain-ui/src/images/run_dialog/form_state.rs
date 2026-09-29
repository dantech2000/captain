use captain_core::model::{ExposedPort, ImageDetail, RestartPolicy};
use captain_core::store::{PortRow, RunForm};
use gpui_kit::component::WindowExt;
use gpui_kit::component::input::InputState;
use gpui_kit::*;

use crate::images::{ImagesState, Started};

/// The Run dialog's fields and the run it starts. The fields hold raw text, and
/// [`RunForm`] checks it when the user clicks Run.
pub struct RunDialog {
    state: Entity<ImagesState>,
    image: String,
    pub(super) name: Entity<InputState>,
    /// One host-port field for each exposed port.
    pub(super) ports: Vec<(ExposedPort, Entity<InputState>)>,
    /// One `KEY=value` field for each variable.
    pub(super) env: Vec<Entity<InputState>>,
    pub(super) restart: RestartPolicy,
    pub(super) auto_remove: bool,
    /// The last validation or engine error.
    pub(super) error: Option<String>,
    pub(super) busy: bool,
    run_task: Option<Task<()>>,
}

impl RunDialog {
    pub fn new(
        state: Entity<ImagesState>,
        image: String,
        detail: &ImageDetail,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let form = RunForm::from_image(image.clone(), detail);
        let name =
            cx.new(|cx| InputState::new(window, cx).placeholder("Leave empty for a random name"));
        let ports = form
            .ports
            .into_iter()
            .map(|row| {
                let input = cx.new(|cx| {
                    InputState::new(window, cx)
                        .placeholder("Not published")
                        .default_value(row.host)
                });
                (row.container, input)
            })
            .collect();
        let env = form
            .env
            .into_iter()
            .map(|line| env_field(line, window, cx))
            .collect();
        Self {
            state,
            image,
            name,
            ports,
            env,
            restart: form.restart,
            auto_remove: form.auto_remove,
            error: None,
            busy: false,
            run_task: None,
        }
    }

    /// The form as the fields hold it now.
    fn form(&self, cx: &App) -> RunForm {
        RunForm {
            image: self.image.clone(),
            name: self.name.read(cx).value().to_string(),
            ports: self
                .ports
                .iter()
                .map(|(container, input)| PortRow {
                    container: container.clone(),
                    host: input.read(cx).value().to_string(),
                })
                .collect(),
            env: self
                .env
                .iter()
                .map(|input| input.read(cx).value().to_string())
                .collect(),
            auto_remove: self.auto_remove,
            restart: self.restart,
        }
    }

    pub(super) fn add_env(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let field = env_field(String::new(), window, cx);
        self.env.push(field);
        cx.notify();
    }

    pub(super) fn remove_env(&mut self, ix: usize, cx: &mut Context<Self>) {
        if ix < self.env.len() {
            self.env.remove(ix);
            cx.notify();
        }
    }

    pub(super) fn set_restart(&mut self, restart: RestartPolicy, cx: &mut Context<Self>) {
        self.restart = restart;
        cx.notify();
    }

    pub(super) fn set_auto_remove(&mut self, auto_remove: bool, cx: &mut Context<Self>) {
        self.auto_remove = auto_remove;
        cx.notify();
    }

    /// Checks the form, then creates and starts the container. The dialog closes when
    /// the engine accepts it, and shows the engine's error when it does not.
    pub(super) fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let spec = match self.form(cx).to_spec() {
            Ok(spec) => spec,
            Err(error) => {
                self.error = Some(error.to_string());
                cx.notify();
                return;
            }
        };
        let state = self.state.read(cx);
        let generation = state.generation();
        let Some(engine) = state.engine.clone() else {
            return;
        };
        self.busy = true;
        self.error = None;
        cx.notify();
        let name = spec.name.clone();
        let state = self.state.clone();
        let run = engine.run_container(spec);
        self.run_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = run.await;
            this.update_in(cx, |this, window, cx| {
                this.busy = false;
                match result {
                    Ok(id) => {
                        let name = name.unwrap_or_else(|| id[..id.len().min(12)].to_string());
                        let started = Started { id, name };
                        state.update(cx, |state, cx| state.set_started(started, generation, cx));
                        window.close_dialog(cx);
                    }
                    Err(error) => {
                        tracing::warn!(%error, "running a container failed");
                        this.error = Some(format!("Run failed: {error}"));
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }
}

fn env_field(line: String, window: &mut Window, cx: &mut Context<RunDialog>) -> Entity<InputState> {
    cx.new(|cx| {
        InputState::new(window, cx)
            .placeholder("KEY=value")
            .default_value(line)
    })
}

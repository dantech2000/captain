use captain_core::store::{BuildForm, DEFAULT_DOCKERFILE};
use futures::StreamExt;
use gpui_kit::component::input::InputState;
use gpui_kit::*;

use crate::images::ImagesState;

/// The most output lines the log view keeps. Older lines drop off the top.
const LOG_LIMIT: usize = 2_000;

/// Where the build is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildStatus {
    Idle,
    Running,
    /// Built and tagged with this reference.
    Built(String),
    Failed(String),
}

/// The Build dialog's fields, the output of the running or last build, and its
/// status. [`BuildForm`] checks the fields when the user clicks Build.
pub struct BuildDialog {
    state: Entity<ImagesState>,
    pub(super) context: Entity<InputState>,
    pub(super) dockerfile: Entity<InputState>,
    pub(super) tag: Entity<InputState>,
    pub(super) target: Entity<InputState>,
    /// One `KEY=value` field for each build argument.
    pub(super) args: Vec<Entity<InputState>>,
    pub(super) log: Vec<SharedString>,
    pub(super) status: BuildStatus,
    /// The last validation error.
    pub(super) error: Option<String>,
    pub(super) scroll: UniformListScrollHandle,
    /// Dropping it drops the output stream, which stops the build.
    build_task: Option<Task<()>>,
}

impl BuildDialog {
    pub fn new(state: Entity<ImagesState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = |placeholder: &'static str, window: &mut Window, cx: &mut Context<Self>| {
            cx.new(|cx| InputState::new(window, cx).placeholder(placeholder))
        };
        Self {
            state,
            context: input("/path/to/project", window, cx),
            dockerfile: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(DEFAULT_DOCKERFILE)
                    .default_value(DEFAULT_DOCKERFILE)
            }),
            tag: input("myapp:dev", window, cx),
            target: input("Leave empty for the last stage", window, cx),
            args: Vec::new(),
            log: Vec::new(),
            status: BuildStatus::Idle,
            error: None,
            scroll: UniformListScrollHandle::new(),
            build_task: None,
        }
    }

    /// The form as the fields hold it now.
    fn form(&self, cx: &App) -> BuildForm {
        let value = |input: &Entity<InputState>| input.read(cx).value().to_string();
        BuildForm {
            context: value(&self.context),
            dockerfile: value(&self.dockerfile),
            tag: value(&self.tag),
            build_args: self.args.iter().map(value).collect(),
            target: value(&self.target),
        }
    }

    pub(super) fn is_running(&self) -> bool {
        self.status == BuildStatus::Running
    }

    pub(super) fn add_arg(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let field = cx.new(|cx| InputState::new(window, cx).placeholder("KEY=value"));
        self.args.push(field);
        cx.notify();
    }

    pub(super) fn remove_arg(&mut self, ix: usize, cx: &mut Context<Self>) {
        if ix < self.args.len() {
            self.args.remove(ix);
            cx.notify();
        }
    }

    /// Checks the form, then starts the build and follows its output.
    pub(super) fn submit(&mut self, cx: &mut Context<Self>) {
        if self.is_running() {
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
        let Some(builder) = self.state.read(cx).builder() else {
            return;
        };
        self.error = None;
        self.log.clear();
        self.status = BuildStatus::Running;
        cx.notify();
        let tag = spec.tag.clone();
        let state = self.state.clone();
        let mut lines = builder.build(&spec);
        self.build_task = Some(cx.spawn(async move |this, cx| {
            let mut failed = None;
            while let Some(line) = lines.next().await {
                let updated = this.update(cx, |this, cx| {
                    match line {
                        Ok(line) => this.push_line(line),
                        Err(error) => failed = Some(error.to_string()),
                    }
                    cx.notify();
                });
                if updated.is_err() || failed.is_some() {
                    break;
                }
            }
            this.update(cx, |this, cx| {
                this.status = match failed {
                    Some(error) => {
                        tracing::warn!(%error, "building an image failed");
                        BuildStatus::Failed(error)
                    }
                    None => {
                        state.update(cx, |state, cx| state.set_notice(format!("Built {tag}"), cx));
                        BuildStatus::Built(tag)
                    }
                };
                cx.notify();
            })
            .ok();
        }));
    }

    fn push_line(&mut self, line: String) {
        if self.log.len() == LOG_LIMIT {
            self.log.remove(0);
        }
        self.log.push(line.into());
        self.scroll.scroll_to_bottom();
    }
}

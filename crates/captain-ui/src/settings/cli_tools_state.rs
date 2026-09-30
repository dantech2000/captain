//! The terminal setup's state: the status, which a background thread reads (it
//! runs a login shell and may ask chezmoi), and Relink. See features 0035 and 0037.

use captain_core::cli_tools::{
    self, CliToolsSettings, SHOWN_TOOLS, SetupSteps, ToolPaths, ToolSource, ToolsStatus,
};
use captain_core::docker_context::CAPTAIN_CONTEXT;
use captain_core::settings::Settings;
use gpui_kit::*;

use super::SettingsView;
use crate::engine_host::captain_socket;

/// What the Terminal section and its sheet show, and the last error.
#[derive(Default)]
pub struct CliToolsCard {
    pub status: Option<ToolsStatus>,
    /// Where each tool comes from in a new terminal, or why Captain cannot tell.
    pub resolved: Option<Result<Vec<(String, ToolSource)>, String>>,
    pub busy: bool,
    pub error: Option<SharedString>,
}

/// The steps, and what the context step needs.
pub struct TerminalFacts {
    pub steps: SetupSteps,
    /// One sentence: all done, or the first thing left.
    pub summary: String,
    /// The docker CLI's default context.
    pub current: String,
    /// Captain Engine's socket, when there is one.
    pub socket: Option<String>,
    /// True if the `captain-engine` context points at that socket.
    pub points_here: bool,
}

impl SettingsView {
    /// The steps, once the background read is done.
    pub(super) fn terminal_facts(&self, settings: &Settings, cx: &App) -> Option<TerminalFacts> {
        let status = self.cli_tools.status.as_ref()?;
        let docker = match &self.cli_tools.resolved {
            Some(Ok(tools)) => tools
                .iter()
                .find(|(name, _)| name == "docker")
                .map(|(_, source)| source),
            _ => None,
        };
        let current = self
            .contexts
            .current
            .clone()
            .unwrap_or_else(|| "default".into());
        let socket = captain_socket(cx);
        let points_here = socket.is_some()
            && self
                .contexts
                .get(CAPTAIN_CONTEXT)
                .is_some_and(|context| context.host == socket);
        let context = points_here && self.contexts.is_current(CAPTAIN_CONTEXT);
        let steps = SetupSteps::of(status, settings.command_line_tools.enabled, docker, context);
        Some(TerminalFacts {
            summary: steps.summary(docker, &current),
            steps,
            current,
            socket,
            points_here,
        })
    }

    /// Reads the state again in the background.
    pub(super) fn refresh_tools(&mut self, cx: &mut Context<Self>) {
        self.run_tools(None, cx);
    }

    /// Installs with `settings` in the background (Relink, a PATH mode change),
    /// then reads the state again.
    pub(super) fn install_tools(&mut self, settings: CliToolsSettings, cx: &mut Context<Self>) {
        self.run_tools(Some(settings), cx);
    }

    fn run_tools(&mut self, install: Option<CliToolsSettings>, cx: &mut Context<Self>) {
        if !cli_tools::SUPPORTED || self.cli_tools.busy {
            return;
        }
        self.cli_tools.busy = true;
        cx.notify();
        let task = cx
            .background_executor()
            .spawn(async move { read_or_install(install) });
        cx.spawn(async move |this, cx| {
            let (status, resolved, errors) = task.await;
            this.update(cx, |view, cx| {
                for error in &errors {
                    tracing::warn!(%error, "cannot set up the command-line tools");
                }
                view.cli_tools = CliToolsCard {
                    status,
                    resolved,
                    busy: false,
                    error: (!errors.is_empty()).then(|| errors.join(" ").into()),
                };
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

type Outcome = (
    Option<ToolsStatus>,
    Option<Result<Vec<(String, ToolSource)>, String>>,
    Vec<String>,
);

fn read_or_install(install: Option<CliToolsSettings>) -> Outcome {
    let Some(paths) = ToolPaths::user() else {
        return (
            None,
            None,
            vec!["Captain cannot find your home folder.".into()],
        );
    };
    let bundle = cli_tools::running_bundle();
    let shell = cli_tools::login_shell();
    let errors = match (&install, &bundle) {
        (Some(settings), Some(bundle)) => {
            cli_tools::install(bundle, &paths, shell.as_deref(), settings.path)
        }
        _ => Vec::new(),
    };
    let status = cli_tools::status(bundle.as_ref(), &paths, shell.as_deref());
    let resolved = shell
        .as_deref()
        .map(|shell| cli_tools::resolve_in_login_shell(shell, &SHOWN_TOOLS, &paths.home));
    (Some(status), resolved, errors)
}

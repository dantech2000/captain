//! The Command-line tools card's state: the status, which a background thread
//! reads (it runs a login shell and may ask chezmoi), and Relink. See feature 0035.

use captain_core::cli_tools::{
    self, CliToolsSettings, SHOWN_TOOLS, ToolPaths, ToolSource, ToolsStatus,
};
use gpui_kit::*;

use super::SettingsView;

/// What the card shows, and the last error.
#[derive(Default)]
pub struct CliToolsCard {
    pub status: Option<ToolsStatus>,
    /// Where each tool comes from in a new terminal, or why Captain cannot tell.
    pub resolved: Option<Result<Vec<(String, ToolSource)>, String>>,
    pub busy: bool,
    pub error: Option<SharedString>,
}

impl SettingsView {
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

//! The sheet's client list: which agents Captain finds (a background read that runs
//! a login shell), and Connect or Remove, which show their step first and then run
//! it. See docs/features/0038-agent-tools.md.

use std::path::PathBuf;
use std::process::Command;

use captain_core::agent_clients::{
    AgentClient, ClientPaths, ClientState, ClientStep, captain_command, connect_step, detect,
    find_commands, login_shell_command, remove_step, run_step,
};
use captain_core::cli_tools::login_shell;
use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::agents_sheet::AgentsSheet;

/// What the client list shows.
#[derive(Default)]
pub struct ClientsCard {
    pub paths: Option<ClientPaths>,
    /// The absolute path of `captain` that clients run.
    pub captain: Option<PathBuf>,
    /// The clients found; `None` while Captain looks.
    pub states: Option<Vec<ClientState>>,
    /// A Connect or Remove that waits for the user to confirm its step.
    pub pending: Option<Pending>,
    pub busy: bool,
    pub error: Option<SharedString>,
}

pub struct Pending {
    pub client: AgentClient,
    pub connect: bool,
    pub step: ClientStep,
}

impl ClientsCard {
    pub fn new() -> Self {
        let paths = ClientPaths::user();
        let captain = paths.as_ref().and_then(|paths| {
            let exe = std::env::current_exe().ok()?;
            captain_command(&paths.home, &exe)
        });
        Self {
            paths,
            captain,
            ..Self::default()
        }
    }
}

impl AgentsSheet {
    /// Looks for the clients again in the background.
    pub(super) fn find_clients(&mut self, cx: &mut Context<Self>) {
        let Some(paths) = self.clients.paths.clone() else {
            self.clients.states = Some(Vec::new());
            self.clients.error = Some("Captain cannot find your home folder.".into());
            return;
        };
        self.clients.states = None;
        cx.notify();
        let task = cx.background_executor().spawn(async move {
            let commands = login_shell()
                .map(|shell| find_commands(&shell, &paths.home))
                .unwrap_or_default();
            detect(&paths, &commands)
        });
        cx.spawn(async move |this, cx| {
            let states = task.await;
            this.update(cx, |sheet, cx| {
                sheet.clients.states = Some(states);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Works out the step of Connect (or Remove) for `state` and shows it.
    pub(super) fn prepare(&mut self, state: &ClientState, connect: bool, cx: &mut Context<Self>) {
        let card = &mut self.clients;
        let Some(paths) = &card.paths else {
            return;
        };
        let step = match (&card.captain, connect) {
            (Some(captain), true) => connect_step(state.client, paths, captain, state.has_command),
            (None, true) => Err("Captain cannot find its captain command.".to_string()),
            (_, false) => remove_step(state.client, paths, state.has_command),
        };
        match step {
            Ok(step) => {
                card.error = None;
                card.pending = Some(Pending {
                    client: state.client,
                    connect,
                    step,
                });
            }
            Err(why) => card.error = Some(why.into()),
        }
        cx.notify();
    }

    /// Runs the step the user confirmed: opens the link, or runs the command or
    /// the edit in the background, then looks at the clients again.
    pub(super) fn run_pending(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(pending) = self.clients.pending.take() else {
            return;
        };
        let name = pending.client.name();
        if let ClientStep::Open(link) = &pending.step {
            cx.open_url(link);
            window.push_notification(
                Notification::info(format!(
                    "{name} asks you to confirm. Then click Check again."
                )),
                cx,
            );
            cx.notify();
            return;
        }
        self.clients.busy = true;
        cx.notify();
        let step = pending.step;
        let task = cx.background_executor().spawn(async move {
            let shell = login_shell();
            let launch = move |argv: &[String]| match &shell {
                Some(shell) => login_shell_command(shell, argv),
                None => {
                    let mut command = Command::new(&argv[0]);
                    command.args(&argv[1..]);
                    command
                }
            };
            run_step(&step, &launch)
        });
        let done = if pending.connect {
            format!("Connected {name} to Captain.")
        } else {
            format!("Removed Captain from {name}.")
        };
        let after = pending.client.after_change();
        cx.spawn_in(window, async move |this, cx| {
            let result = task.await;
            this.update_in(cx, |sheet, window, cx| {
                sheet.clients.busy = false;
                match result {
                    Ok(()) => {
                        let message = match after {
                            Some(after) => format!("{done} {after}"),
                            None => done,
                        };
                        tracing::info!(%message, "agent client changed");
                        window.push_notification(Notification::success(message), cx);
                    }
                    Err(why) => {
                        tracing::warn!(%why, "cannot change the agent client");
                        sheet.clients.error = Some(why.into());
                    }
                }
                sheet.find_clients(cx);
            })
            .ok();
        })
        .detach();
    }
}

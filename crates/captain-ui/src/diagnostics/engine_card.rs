use captain_core::HostStatus;
use gpui_kit::component::Sizable;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::*;

use crate::engine_host::{HostModel, HostSummary};
use crate::help::HelpExt;
use crate::shell::EngineState;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, settings_card, settings_row, text_button};
use crate::workspace::{Connection, Workspace};

/// The engine and its state at the top of Diagnostics. Captain Engine gets Start,
/// Stop and Restart, or Set up; another engine gets "Use Captain Engine" when
/// Captain Engine is available. See feature 0016.
pub fn render(
    workspace: &Workspace,
    host: Option<&HostSummary>,
    captain: Option<Entity<HostModel>>,
    palette: &Palette,
) -> Div {
    let engine = EngineState::of(workspace, host, palette);
    let busy = host.is_some_and(|host| host.status.is_busy());
    let state = if busy {
        format!("{}\u{2026}", engine.state)
    } else {
        engine.state.to_string()
    };
    let label = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(div().size(px(8.)).rounded_full().bg(engine.color))
        .child(format!("{} \u{00b7} {state}", engine.engine));
    let (note, controls) = match host {
        Some(host) => (captain_note(&host.status), captain_controls(host, palette)),
        None => (other_note(workspace), other_controls(captain, palette)),
    };
    let row = settings_row(label, Some(note.into()), controls, palette);
    settings_card("Engine", [row.into_any_element()], palette)
}

fn captain_note(status: &HostStatus) -> String {
    match status {
        HostStatus::Running => "Containers run in Captain Engine's virtual machine.".into(),
        HostStatus::Starting => "Captain Engine is starting its virtual machine.".into(),
        HostStatus::Stopping => "Captain Engine is stopping its virtual machine.".into(),
        HostStatus::Stopped => "Start it to run containers.".into(),
        HostStatus::NotCreated => "Set it up to download and create its virtual machine.".into(),
        HostStatus::Failed(why) | HostStatus::NotInstalled(why) => why.clone(),
    }
}

/// Start (or Set up) while it is stopped, Stop and Restart while it runs, and a
/// spinner beside Stop and Restart while it starts or stops.
fn captain_controls(host: &HostSummary, palette: &Palette) -> Div {
    let status = &host.status;
    let control = host.can_control;
    let model = host.model.clone();
    let row = div().flex().items_center().gap(px(8.));
    if status.can_start() || matches!(status, HostStatus::NotInstalled(_)) {
        let (id, label, help) = if *status == HostStatus::NotCreated {
            (
                "diagnostics-engine-setup",
                "Set up",
                "Set up Captain Engine: download and create its virtual machine.",
            )
        } else {
            ("diagnostics-engine-start", "Start", "Start Captain Engine.")
        };
        let enabled = control && status.can_start();
        return row.child(
            text_button(
                id,
                label,
                ButtonTone::Accent,
                enabled,
                palette,
                move |_, _, cx| model.update(cx, |model, cx| model.start(cx)),
            )
            .help(help),
        );
    }
    let busy = status.is_busy();
    let restart = model.clone();
    row.children(busy.then(|| {
        div()
            .id("diagnostics-engine-busy")
            .child(Spinner::new().xsmall().color(palette.text2))
            .help(format!(
                "Captain Engine is {}.",
                status.label().to_lowercase()
            ))
    }))
    .child(
        text_button(
            "diagnostics-engine-stop",
            "Stop",
            ButtonTone::Accent,
            control && !busy && status.can_stop(),
            palette,
            move |_, _, cx| model.update(cx, |model, cx| model.stop(cx)).detach(),
        )
        .help("Stop Captain Engine. Running containers stop with it."),
    )
    .child(
        text_button(
            "diagnostics-engine-restart",
            "Restart",
            ButtonTone::Accent,
            control && status.is_running(),
            palette,
            move |_, _, cx| restart.update(cx, |model, cx| model.restart(cx)),
        )
        .help("Stop Captain Engine and start it again, for example to use saved settings. Running containers stop with it."),
    )
}

/// Where the other engine is, from the connection.
fn other_note(workspace: &Workspace) -> String {
    match workspace.connection() {
        Connection::Connected(info) => {
            format!(
                "{} \u{00b7} Docker {}. Captain does not start or stop it.",
                info.endpoint, info.version
            )
        }
        Connection::Connecting => "Captain is connecting to the engine.".into(),
        Connection::Failed(error) => format!("Captain cannot reach the engine: {error}"),
    }
}

/// "Use Captain Engine", as in the Settings engine menu, when the app has it and
/// it runs on this platform.
fn other_controls(captain: Option<Entity<HostModel>>, palette: &Palette) -> Div {
    div().children(captain.map(|model| {
        text_button(
            "diagnostics-engine-use-captain",
            "Use Captain Engine",
            ButtonTone::Accent,
            true,
            palette,
            move |_, _, cx| model.update(cx, |model, cx| model.use_captain(cx)),
        )
        .help("Switch to Captain Engine: Captain connects to it, starts it if it is stopped, or shows its setup.")
    }))
}

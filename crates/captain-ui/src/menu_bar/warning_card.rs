//! The warning card: the worst problem, what caused it, and buttons that fix it.

use captain_core::format::bytes_label;
use captain_core::model::ContainerAction;
use captain_core::problems::{Problem, raised_memory};
use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::controls::{Tone, button};
use super::float_log::open_float_log;
use crate::diagnostics::{diagnostics_model, run_fix};
use crate::theme::Palette;
use crate::workspace::Workspace;

pub fn render(problem: &Problem, workspace: &Entity<Workspace>, palette: &Palette) -> Div {
    let (name, text) = sentence(problem);
    let buttons = div().flex().flex_wrap().gap(px(6.));
    let buttons = match problem {
        Problem::EngineFailed { fix, .. }
        | Problem::FailedCheck(captain_core::diagnostics::Check { fix: Some(fix), .. }) => {
            let fix = fix.clone();
            buttons.child(button(
                "popover-fix",
                fix.label(),
                "Run the fix that Diagnostics suggests for this problem.",
                Tone::Primary,
                false,
                palette,
                move |window, cx| {
                    let engine_dir = diagnostics_model(cx).and_then(|m| m.read(cx).engine_dir());
                    run_fix(&fix, engine_dir.as_ref(), window, cx);
                },
            ))
        }
        Problem::FailedCheck(_) => buttons,
        Problem::OutOfMemory {
            id, name, limit, ..
        } => {
            let raise = (*limit > 0).then(|| raise_button(id, name, *limit, workspace, palette));
            buttons
                .children(raise)
                .child(logs_button(id, name, workspace, palette))
                .child(action_button(
                    id,
                    name,
                    ContainerAction::Stop,
                    workspace,
                    palette,
                ))
        }
        Problem::Restarting { id, name } => buttons
            .child(logs_button(id, name, workspace, palette))
            .child(action_button(
                id,
                name,
                ContainerAction::Stop,
                workspace,
                palette,
            )),
        Problem::Unhealthy { id, name } => buttons
            .child(logs_button(id, name, workspace, palette))
            .child(action_button(
                id,
                name,
                ContainerAction::Restart,
                workspace,
                palette,
            )),
    };
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .mx(px(12.))
        .mb(px(10.))
        .p(px(12.))
        .rounded(px(11.))
        .bg(palette.red.opacity(0.1))
        .border_1()
        .border_color(palette.red.opacity(0.35))
        .child(div().text_size(px(12.5)).child(styled_sentence(name, text)))
        .child(buttons)
}

/// The sentence, with the container's name in bold at its start.
fn styled_sentence(name: Option<String>, rest: String) -> StyledText {
    let bold = name.as_ref().map_or(0, String::len);
    let text = format!("{}{rest}", name.unwrap_or_default());
    let style = HighlightStyle {
        font_weight: Some(FontWeight::BOLD),
        ..Default::default()
    };
    StyledText::new(text).with_highlights((bold > 0).then_some((0..bold, style)))
}

/// The container's name and the rest of the sentence.
fn sentence(problem: &Problem) -> (Option<String>, String) {
    match problem {
        Problem::EngineFailed { why, .. } => (None, format!("Captain Engine did not start: {why}")),
        Problem::OutOfMemory {
            name,
            limit,
            restarts,
            ..
        } => {
            let times = match restarts {
                1 => "once".to_string(),
                n => format!("{n} times"),
            };
            let cause = if *limit > 0 {
                format!("out of memory at {}", bytes_label(*limit as u64))
            } else {
                "the engine ran out of memory".to_string()
            };
            (
                Some(name.clone()),
                format!(" keeps restarting: {cause}, restarted {times}."),
            )
        }
        Problem::Restarting { name, .. } => (
            Some(name.clone()),
            " keeps restarting: its process exits with an error. The logs say why.".into(),
        ),
        Problem::Unhealthy { name, .. } => (
            Some(name.clone()),
            " fails its health check. The logs say why; a restart often helps.".into(),
        ),
        Problem::FailedCheck(check) => (None, format!("{}: {}", check.id.title(), check.detail)),
    }
}

fn raise_button(
    id: &str,
    name: &str,
    limit: i64,
    workspace: &Entity<Workspace>,
    palette: &Palette,
) -> Stateful<Div> {
    let raised = raised_memory(limit);
    let label = format!("Raise to {}", bytes_label(raised as u64));
    let help = format!(
        "Give {name} {} of memory instead of {} (docker update --memory).",
        bytes_label(raised as u64),
        bytes_label(limit as u64)
    );
    let (id, name, workspace) = (id.to_string(), name.to_string(), workspace.clone());
    button(
        "popover-raise",
        label,
        help,
        Tone::Primary,
        false,
        palette,
        move |window, cx| {
            let Some(engine) = workspace.read(cx).engine() else {
                return;
            };
            let update = engine.update_memory(&id, raised);
            let name = name.clone();
            window
                .spawn(cx, async move |cx| {
                    let note = match update.await {
                        Ok(()) => Notification::success(format!(
                            "{name} can now use {}.",
                            bytes_label(raised as u64)
                        )),
                        Err(error) => {
                            tracing::warn!(%error, "cannot raise the memory limit");
                            Notification::error(format!(
                                "Captain could not raise the limit: {error}"
                            ))
                        }
                    };
                    cx.update(|window, cx| window.push_notification(note, cx))
                        .ok();
                })
                .detach();
        },
    )
}

fn logs_button(
    id: &str,
    name: &str,
    workspace: &Entity<Workspace>,
    palette: &Palette,
) -> Stateful<Div> {
    let (id, name, workspace) = (id.to_string(), name.to_string(), workspace.clone());
    let help = format!("Open {name}'s log in a small window that stays on top.");
    button(
        "popover-show-logs",
        "Show logs",
        help,
        Tone::Plain,
        false,
        palette,
        move |_, cx| {
            open_float_log(workspace.clone(), id.clone(), name.clone(), cx);
        },
    )
}

fn action_button(
    id: &str,
    name: &str,
    action: ContainerAction,
    workspace: &Entity<Workspace>,
    palette: &Palette,
) -> Stateful<Div> {
    let label = format!("{} {name}", action.label());
    let help = match action {
        ContainerAction::Stop => format!("Stop {name}, so it stops restarting."),
        _ => format!("Restart {name}."),
    };
    let (id, workspace) = (id.to_string(), workspace.clone());
    button(
        "popover-container-action",
        label,
        help,
        Tone::Quiet,
        false,
        palette,
        move |_, cx| {
            workspace.update(cx, |workspace, cx| {
                workspace.run_action(id.clone(), action, cx)
            });
        },
    )
}

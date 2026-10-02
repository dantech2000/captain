//! The worst problem: a status line in the problem's words, then the items that fix
//! it. It replaces the popover's warning card. See feature 0032.

use captain_core::diagnostics::{Check, Fix};
use captain_core::format::bytes_label;
use captain_core::problems::{ContainerFix, Problem};

use super::dot::Light;
use super::menu_model::{TrayCommand, TrayItem};

/// The stop light for `problem`, for its menu line and the dot on the menu bar
/// icon: amber for an unhealthy container, red for everything else.
pub fn problem_light(problem: &Problem) -> Light {
    match problem {
        Problem::Unhealthy { .. } => Light::Amber,
        _ => Light::Red,
    }
}

/// The problem's line, then its fixes.
pub fn problem_items(problem: &Problem) -> Vec<TrayItem> {
    let light = problem_light(problem);
    let mut items = vec![TrayItem::Status {
        label: problem.line(),
        light,
    }];
    match problem {
        Problem::EngineFailed { fix, .. } | Problem::FailedCheck(Check { fix: Some(fix), .. }) => {
            items.push(TrayItem::command(
                fix_label(fix),
                TrayCommand::RunFix(fix.clone()),
            ));
        }
        Problem::FailedCheck(_) => {}
        Problem::OutOfMemory { id, name, .. }
        | Problem::Restarting { id, name }
        | Problem::Unhealthy { id, name } => {
            items.extend(
                problem
                    .container_fixes()
                    .into_iter()
                    .map(|fix| fix_item(id, name, fix)),
            );
        }
    }
    items
}

/// Raise Memory, Show Logs, then Stop to end a crash loop, or Restart for a failed
/// health check.
fn fix_item(id: &str, name: &str, fix: ContainerFix) -> TrayItem {
    match fix {
        ContainerFix::RaiseMemory(bytes) => TrayItem::command(
            format!("Raise Memory to {}", bytes_label(bytes)),
            TrayCommand::RaiseMemory {
                id: id.into(),
                name: name.into(),
                bytes,
            },
        ),
        ContainerFix::ShowLogs => TrayItem::command(
            "Show Logs in a Window",
            TrayCommand::FloatLog {
                id: id.into(),
                name: name.into(),
            },
        ),
        ContainerFix::Run(action) => TrayItem::command(
            format!("{} {name}", action.label()),
            TrayCommand::Container {
                id: id.into(),
                action,
            },
        ),
    }
}

/// The fix's button label in Title Case, as menu items are.
fn fix_label(fix: &Fix) -> &'static str {
    match fix {
        Fix::StartEngine => "Start Captain Engine",
        Fix::RestartEngine => "Restart Captain Engine",
        Fix::ShowEngineFiles => "Show Engine Files",
        Fix::CopyCommand(_) => "Copy Command",
        Fix::OpenSettingsFile => "Open settings.json",
    }
}

#[cfg(test)]
mod tests;

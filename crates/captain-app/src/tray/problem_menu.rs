//! The worst problem: a status line in the problem's words, then the items that fix
//! it. It replaces the popover's warning card. See feature 0032.

use captain_core::diagnostics::{Check, Fix};
use captain_core::format::bytes_label;
use captain_core::model::ContainerAction;
use captain_core::problems::{Problem, raised_memory};

use super::dot::Light;
use super::menu_model::{TrayCommand, TrayItem};

/// The problem's line, then its fixes.
pub fn problem_items(problem: &Problem) -> Vec<TrayItem> {
    let light = match problem {
        Problem::Unhealthy { .. } => Light::Amber,
        _ => Light::Red,
    };
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
        Problem::OutOfMemory {
            id, name, limit, ..
        } => {
            // With no limit the engine itself ran out; a higher limit does not help.
            if *limit > 0 {
                let bytes = raised_memory(*limit).max(0) as u64;
                items.push(TrayItem::command(
                    format!("Raise Memory to {}", bytes_label(bytes)),
                    TrayCommand::RaiseMemory {
                        id: id.clone(),
                        name: name.clone(),
                        bytes,
                    },
                ));
            }
            items.extend(container_fixes(id, name, ContainerAction::Stop));
        }
        Problem::Restarting { id, name } => {
            items.extend(container_fixes(id, name, ContainerAction::Stop));
        }
        Problem::Unhealthy { id, name } => {
            items.extend(container_fixes(id, name, ContainerAction::Restart));
        }
    }
    items
}

/// Show Logs, then Stop to end a crash loop, or Restart for a failed health check.
fn container_fixes(id: &str, name: &str, action: ContainerAction) -> [TrayItem; 2] {
    [
        TrayItem::command(
            "Show Logs in a Window",
            TrayCommand::FloatLog {
                id: id.into(),
                name: name.into(),
            },
        ),
        TrayItem::command(
            format!("{} {name}", action.label()),
            TrayCommand::Container {
                id: id.into(),
                action,
            },
        ),
    ]
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

use captain_core::model::{Container, ContainerDetail, ContainerState, Health};
use gpui_kit::*;

use crate::theme::Palette;

/// The line under a card's title, and its color: why a container stopped, its
/// health, or how long it runs. `exits` counts its exits in the last two minutes.
pub fn card_note(
    container: &Container,
    detail: Option<&ContainerDetail>,
    exits: usize,
    palette: &Palette,
) -> (String, Hsla) {
    let stopped = matches!(
        container.state,
        ContainerState::Restarting | ContainerState::Exited | ContainerState::Dead
    );
    if stopped && let Some(detail) = detail.filter(|d| d.exit_code != 0 || d.oom_killed) {
        let mut note = format!("Exit {}", detail.exit_code);
        if detail.oom_killed {
            note.push_str(" · out of memory");
        }
        if exits > 1 {
            note.push_str(&format!(" · {exits} times in 2 min"));
        }
        return (note, palette.red);
    }
    match container.state {
        ContainerState::Restarting => ("Restarting".into(), palette.red),
        ContainerState::Running => match container.health {
            Some(Health::Unhealthy) => ("Health: failing".into(), palette.red),
            Some(Health::Starting) => ("Health: starting".into(), palette.warn_text),
            Some(Health::Healthy) => (
                format!("Healthy · up {}", container.uptime_label()),
                palette.text2,
            ),
            None => (format!("Up {}", container.uptime_label()), palette.text2),
        },
        ContainerState::Paused => ("Paused: its processes are frozen".into(), palette.warn_text),
        ContainerState::Created => ("Created, not started".into(), palette.text2),
        _ => (container.status.clone(), palette.text2),
    }
}

/// The status bar sentence for a card's state pill.
pub fn state_help(container: &Container, detail: Option<&ContainerDetail>, name: &str) -> String {
    let limit = detail
        .map(|d| d.memory_limit)
        .filter(|limit| *limit > 0)
        .map(captain_core::format::bytes_label);
    match container.state {
        ContainerState::Restarting | ContainerState::Exited
            if detail.is_some_and(|d| d.oom_killed) =>
        {
            match limit {
                Some(limit) => format!(
                    "{name} used more than its {limit} memory limit, so the kernel stopped it."
                ),
                None => format!("The engine ran out of memory and stopped {name}."),
            }
        }
        ContainerState::Restarting => {
            format!("{name} stopped and its restart policy starts it again.")
        }
        ContainerState::Running => match container.health {
            Some(Health::Unhealthy) => format!("{name} runs, but its health check fails."),
            Some(Health::Healthy) => format!("{name} runs and passes its health check."),
            _ => format!("{name} runs. {}.", container.status),
        },
        ContainerState::Paused => {
            format!("{name} is paused. Its processes are frozen, not stopped.")
        }
        _ => format!(
            "{name} is {}. {}.",
            container.state.label(),
            container.status
        ),
    }
}

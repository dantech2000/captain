//! The problems that the menu bar menu warns about and the Dock badge counts, and
//! the one wording for them. See docs/features/0032-menu-bar-popover.md.

use std::collections::HashMap;

use crate::diagnostics::{Check, CheckState, Fix};
use crate::format::bytes_label;
use crate::model::{Container, ContainerState, Health};
use crate::store::Crash;

/// Why a restarting container stopped, from `inspect`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExitFacts {
    /// The kernel killed the last run for using too much memory.
    pub oom_killed: bool,
    /// The memory limit in bytes. 0 means no limit.
    pub memory_limit: i64,
    pub restart_count: i64,
}

/// The one problem the menu bar menu shows, with its fixes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// Captain Engine did not start. `fix` comes from the failed diagnostics check.
    EngineFailed {
        why: String,
        fix: Fix,
    },
    /// A container keeps restarting because it runs out of memory. `limit` is 0 when
    /// the container has no limit and the engine itself ran out.
    OutOfMemory {
        id: String,
        name: String,
        limit: i64,
        restarts: i64,
    },
    Restarting {
        id: String,
        name: String,
    },
    Unhealthy {
        id: String,
        name: String,
    },
    FailedCheck(Check),
}

impl Problem {
    /// The container the problem is about, if any.
    pub fn container(&self) -> Option<(&str, &str)> {
        match self {
            Self::OutOfMemory { id, name, .. }
            | Self::Restarting { id, name }
            | Self::Unhealthy { id, name } => Some((id, name)),
            Self::EngineFailed { .. } | Self::FailedCheck(_) => None,
        }
    }

    /// The problem as a sentence: the container's name, if any, and the rest of
    /// the sentence.
    pub fn sentence(&self) -> (Option<&str>, String) {
        match self {
            Self::EngineFailed { why, .. } => {
                (None, format!("Captain Engine did not start: {why}"))
            }
            Self::OutOfMemory {
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
                    Some(name),
                    format!(" keeps restarting: {cause}, restarted {times}."),
                )
            }
            Self::Restarting { name, .. } => (
                Some(name),
                " keeps restarting: its process exits with an error. The logs say why.".into(),
            ),
            Self::Unhealthy { name, .. } => (
                Some(name),
                " fails its health check. The logs say why; a restart often helps.".into(),
            ),
            Self::FailedCheck(check) => (None, format!("{}: {}", check.id.title(), check.detail)),
        }
    }

    /// [`Problem::sentence`] as one plain line, for the tray menu.
    pub fn line(&self) -> String {
        let (name, rest) = self.sentence();
        format!("{}{rest}", name.unwrap_or_default())
    }
}

/// True while a container is restarting, crashed within the last minute (`crashed`,
/// from the event stream: a crash loop spends most of its time running), or its
/// health check fails.
pub fn needs_attention(container: &Container, crashed: bool) -> bool {
    crashed
        || container.state == ContainerState::Restarting
        || (container.state == ContainerState::Running
            && container.health == Some(Health::Unhealthy))
}

/// The number for the Dock badge: failed checks, and containers that need attention.
pub fn problem_count(
    failed_checks: usize,
    containers: &[Container],
    crashed: impl Fn(&str) -> bool,
) -> usize {
    failed_checks
        + containers
            .iter()
            .filter(|c| needs_attention(c, crashed(&c.id)))
            .count()
}

/// The limit the menu offers when a container runs out of memory: twice the old
/// one, at least 512 MB, like the project page and the map.
pub fn raised_memory(limit: i64) -> i64 {
    limit.saturating_mul(2)
}

/// The problem the menu shows. A failed engine comes first, because
/// nothing else works without it. Then containers, worst first: out of memory,
/// restarting, unhealthy. Failed diagnostics checks come last. `facts` holds what
/// `inspect` said about restarting containers; `crash` gives a container's recent
/// crash from the event stream, if any.
pub fn first_problem(
    engine_failure: Option<&str>,
    checks: &[Check],
    containers: &[Container],
    facts: &HashMap<String, ExitFacts>,
    crash: &dyn Fn(&str) -> Option<Crash>,
) -> Option<Problem> {
    let failed = || checks.iter().filter(|c| c.state == CheckState::Failed);
    if let Some(why) = engine_failure {
        let fix = failed()
            .find_map(|check| check.fix.clone())
            .unwrap_or(Fix::RestartEngine);
        return Some(Problem::EngineFailed {
            why: why.to_string(),
            fix,
        });
    }
    let restarting = || {
        containers
            .iter()
            .filter(|c| c.state == ContainerState::Restarting || crash(&c.id).is_some())
    };
    let out_of_memory = restarting().find_map(|c| {
        // Docker clears OOMKilled at each start; the `oom` event does not go away.
        let oom_event = crash(&c.id).is_some_and(|crash| crash.out_of_memory);
        let facts = facts
            .get(&c.id)
            .filter(|facts| facts.oom_killed || oom_event)?;
        Some(Problem::OutOfMemory {
            id: c.id.clone(),
            name: c.name.clone(),
            limit: facts.memory_limit,
            restarts: facts.restart_count,
        })
    });
    out_of_memory
        .or_else(|| {
            restarting().next().map(|c| Problem::Restarting {
                id: c.id.clone(),
                name: c.name.clone(),
            })
        })
        .or_else(|| {
            containers
                .iter()
                .find(|c| needs_attention(c, false))
                .map(|c| Problem::Unhealthy {
                    id: c.id.clone(),
                    name: c.name.clone(),
                })
        })
        .or_else(|| failed().next().cloned().map(Problem::FailedCheck))
}

#[cfg(test)]
mod tests;

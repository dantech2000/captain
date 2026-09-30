//! The problems that the menu bar menu warns about and the Dock badge counts, and
//! the one wording for them. See docs/features/0032-menu-bar-popover.md.

use std::collections::HashMap;

use crate::diagnostics::{Check, CheckState, Fix};
use crate::format::bytes_label;
use crate::model::{Container, ContainerAction, ContainerDetail, ContainerState, Health};
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

impl ExitFacts {
    /// What `inspect` said about the last run.
    pub fn of(detail: &ContainerDetail) -> Self {
        Self {
            oom_killed: detail.oom_killed,
            memory_limit: i64::try_from(detail.memory_limit).unwrap_or(i64::MAX),
            restart_count: detail.restart_count,
        }
    }
}

/// What Captain offers for a container's problem, best first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerFix {
    /// Raise the memory limit to this many bytes.
    RaiseMemory(u64),
    ShowLogs,
    Run(ContainerAction),
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

    /// The fixes for a container's problem: Raise Memory when a limit ran out,
    /// then the logs, then Stop to end a crash loop or Restart for a failed health
    /// check. Other problems have none.
    pub fn container_fixes(&self) -> Vec<ContainerFix> {
        let mut fixes = Vec::new();
        let action = match self {
            // With no limit the engine itself ran out; a higher limit does not help.
            Self::OutOfMemory { limit, .. } => {
                if *limit > 0 {
                    fixes.push(ContainerFix::RaiseMemory(raised_memory(*limit)));
                }
                ContainerAction::Stop
            }
            Self::Restarting { .. } => ContainerAction::Stop,
            Self::Unhealthy { .. } => ContainerAction::Restart,
            Self::EngineFailed { .. } | Self::FailedCheck(_) => return fixes,
        };
        fixes.extend([ContainerFix::ShowLogs, ContainerFix::Run(action)]);
        fixes
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

/// The limit the menu offers when a container runs out of memory, in bytes. It uses
/// the rule of the project page and the map: twice the old one, at least 512 MB.
pub fn raised_memory(limit: i64) -> u64 {
    crate::project_map::raised_memory(limit.max(0) as u64)
}

/// The problem the menu shows. A failed engine comes first, because
/// nothing else works without it. Then containers, worst first (see
/// [`container_problems`]). Failed diagnostics checks come last.
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
    container_problems(containers, facts, crash)
        .into_iter()
        .next()
        .or_else(|| failed().next().cloned().map(Problem::FailedCheck))
}

/// Every container that needs attention, worst first: out of memory, restarting,
/// unhealthy. `facts` holds what `inspect` said about restarting containers; `crash`
/// gives a container's recent crash from the event stream, if any.
pub fn container_problems(
    containers: &[Container],
    facts: &HashMap<String, ExitFacts>,
    crash: &dyn Fn(&str) -> Option<Crash>,
) -> Vec<Problem> {
    let (restarting, others): (Vec<&Container>, Vec<&Container>) = containers
        .iter()
        .partition(|c| c.state == ContainerState::Restarting || crash(&c.id).is_some());
    let out_of_memory = |c: &Container| {
        // Docker clears OOMKilled at each start; the `oom` event does not go away.
        let oom_event = crash(&c.id).is_some_and(|crash| crash.out_of_memory);
        facts
            .get(&c.id)
            .filter(|facts| facts.oom_killed || oom_event)
            .copied()
    };
    let (memory, plain): (Vec<&Container>, Vec<&Container>) = restarting
        .into_iter()
        .partition(|c| out_of_memory(c).is_some());
    let memory = memory.into_iter().filter_map(|c| {
        let facts = out_of_memory(c)?;
        Some(Problem::OutOfMemory {
            id: c.id.clone(),
            name: c.name.clone(),
            limit: facts.memory_limit,
            restarts: facts.restart_count,
        })
    });
    let plain = plain.into_iter().map(|c| Problem::Restarting {
        id: c.id.clone(),
        name: c.name.clone(),
    });
    let unhealthy = others
        .into_iter()
        .filter(|c| needs_attention(c, false))
        .map(|c| Problem::Unhealthy {
            id: c.id.clone(),
            name: c.name.clone(),
        });
    memory.chain(plain).chain(unhealthy).collect()
}

#[cfg(test)]
mod tests;

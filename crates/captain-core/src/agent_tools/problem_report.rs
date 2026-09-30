//! The `container_problems` answer: what Captain's menu bar would warn about, for
//! every container at once, with the facts from `inspect` and the fixes Captain
//! offers.

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Serialize;

use super::container_rows::group_of;
use super::untrusted::plain;
use crate::format::bytes_label;
use crate::model::{Container, ContainerDetail};
use crate::problems::{ContainerFix, ExitFacts, Problem, container_problems};
use crate::store::Crash;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ProblemReport {
    /// Why Captain Engine does not run, when it does not. Nothing else works then.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// Worst first: out of memory, restarting, unhealthy.
    pub problems: Vec<ContainerProblem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ContainerProblem {
    pub container: String,
    /// The Compose project, or `k8s:<namespace>`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// out_of_memory, restarting, or unhealthy.
    pub kind: String,
    /// The problem in Captain's words.
    pub summary: String,
    /// The exit code of the last run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart_count: Option<i64>,
    /// The memory limit, for example `256 MB`; absent without a limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit: Option<String>,
    /// What Captain offers, best first.
    pub fixes: Vec<String>,
}

/// The problems of `containers`. `details` holds `inspect` of the containers that
/// restart, crashed, or fail a health check; `crash` gives recent crashes from the
/// event stream.
pub fn problem_report(
    engine: Option<String>,
    containers: &[Container],
    details: &HashMap<String, ContainerDetail>,
    crash: &dyn Fn(&str) -> Option<Crash>,
) -> ProblemReport {
    let facts: HashMap<String, ExitFacts> = details
        .iter()
        .map(|(id, detail)| (id.clone(), ExitFacts::of(detail)))
        .collect();
    let problems = container_problems(containers, &facts, crash)
        .iter()
        .filter_map(|problem| {
            let (id, _) = problem.container()?;
            let container = containers.iter().find(|c| c.id == id)?;
            Some(row(problem, container, details.get(id)))
        })
        .collect();
    ProblemReport { engine, problems }
}

fn row(
    problem: &Problem,
    container: &Container,
    detail: Option<&ContainerDetail>,
) -> ContainerProblem {
    let kind = match problem {
        Problem::OutOfMemory { .. } => "out_of_memory",
        Problem::Restarting { .. } => "restarting",
        _ => "unhealthy",
    };
    let fixes = problem
        .container_fixes()
        .into_iter()
        .map(|fix| match fix {
            ContainerFix::RaiseMemory(bytes) => {
                format!("Raise its memory limit to {}.", bytes_label(bytes))
            }
            ContainerFix::ShowLogs => "Read its logs with the logs tool.".into(),
            ContainerFix::Run(action) => format!("{} it.", action.label()),
        })
        .collect();
    ContainerProblem {
        container: container.display_name(),
        project: group_of(container),
        kind: kind.into(),
        summary: problem.line(),
        exit_code: detail.map(|d| d.exit_code),
        restart_count: detail.map(|d| d.restart_count),
        memory_limit: detail
            .map(|d| d.memory_limit)
            .filter(|limit| *limit > 0)
            .map(bytes_label),
        fixes,
    }
}

impl ProblemReport {
    /// The problems as lines, for clients that show only text.
    pub fn text(&self) -> String {
        let mut lines: Vec<String> = self.engine.iter().cloned().collect();
        for problem in &self.problems {
            let mut line = problem.summary.clone();
            if let Some(code) = problem.exit_code {
                line.push_str(&format!(" Exit code {code}."));
            }
            line.push_str(" Fixes: ");
            line.push_str(&problem.fixes.join(" "));
            lines.push(plain(&line));
        }
        if lines.is_empty() {
            return "No problems: no container restarts, crashed lately, or fails its health check.".into();
        }
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests;

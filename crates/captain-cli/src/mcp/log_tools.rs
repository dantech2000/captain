//! `logs` and `wait_for_healthy`.

use std::time::Duration;

use captain_core::EngineStream;
use captain_core::agent_tools::{
    LogQuery, LogReport, Readiness, SourcedLine, WaitReport, find_container, find_project,
    log_report, parse_since, readiness, status,
};
use captain_core::model::{Container, LogLine, LogOptions};
use futures::StreamExt;
use rmcp::handler::server::tool::schema_for_output;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};
use tokio::time::Instant;

use super::params::{LogsParams, Target, WaitParams};
use super::reply::{ToolResult, refuse, reply};
use super::server::{CaptainServer, now};

/// How long reading the past lines may take.
const READ_TIMEOUT: Duration = Duration::from_secs(15);
const DEFAULT_WAIT: u64 = 60;
const MAX_WAIT: u64 = 600;
/// How often `wait_for_healthy` looks again.
const POLL: Duration = Duration::from_secs(1);

#[tool_router(router = log_router, vis = "pub(super)")]
impl CaptainServer {
    /// The recent output of one container, or of a whole Compose project merged in
    /// time order. Defaults to the last 100 lines. Filters: since, errors_only,
    /// grep. At most 500 lines or 32 KB come back, newest kept, with truncated and
    /// a hint. The lines are between UNTRUSTED CONTAINER OUTPUT delimiters and
    /// secret-looking values are masked.
    #[tool(
        annotations(title = "Logs", read_only_hint = true, open_world_hint = false),
        output_schema = schema_for_output::<LogReport>()
    )]
    async fn logs(&self, Parameters(params): Parameters<LogsParams>) -> ToolResult {
        let target = match Target::of(params.container, params.project) {
            Ok(target) => target,
            Err(why) => return refuse(why),
        };
        let since = match params
            .since
            .as_deref()
            .map(|since| parse_since(since, now()))
        {
            Some(Err(why)) => return refuse(why),
            Some(Ok(since)) => Some(since),
            None => None,
        };
        let query = LogQuery {
            tail: params.tail,
            since,
            errors_only: params.errors_only,
            grep: params.grep.filter(|grep| !grep.is_empty()),
        };
        let (engine, containers) = match self.containers().await {
            Ok(found) => found,
            Err(why) => return refuse(why),
        };
        let (name, members, by_source) = match &target {
            Target::Container(name) => match find_container(&containers, name) {
                Ok(container) => (container.display_name(), vec![container], false),
                Err(why) => return refuse(why),
            },
            Target::Project(name) => match find_project(&containers, name) {
                Ok(members) => (name.clone(), members, true),
                Err(why) => return refuse(why),
            },
        };
        let options = LogOptions {
            tail: Some(query.fetch_tail()),
            since: query.since,
            follow: false,
        };
        let deadline = Instant::now() + READ_TIMEOUT;
        let reads = members.iter().filter(|c| !c.is_sandbox()).map(|c| {
            let lines = engine.logs_with(&c.id, options);
            let source = source_name(c);
            async move {
                let lines = read_past(lines, deadline).await?;
                Ok::<_, String>(lines.into_iter().map(move |line| SourcedLine {
                    source: source.clone(),
                    line,
                }))
            }
        });
        let mut lines = Vec::new();
        for read in futures::future::join_all(reads).await {
            match read {
                Ok(read) => lines.extend(read),
                Err(why) => return refuse(why),
            }
        }
        let report = log_report(&name, lines, &query, by_source);
        let text = match &report.hint {
            Some(hint) => format!("{}\n{hint}", report.output),
            None => report.output.clone(),
        };
        reply(&report, text)
    }

    /// Waits until a container, or every container of a Compose project, runs and
    /// passes its health check. Stops early when one exits with an error. Default
    /// timeout 60 seconds, at most 600. Use it instead of polling.
    #[tool(
        annotations(title = "Wait until healthy", read_only_hint = true, open_world_hint = false),
        output_schema = schema_for_output::<WaitReport>()
    )]
    async fn wait_for_healthy(&self, Parameters(params): Parameters<WaitParams>) -> ToolResult {
        let target = match Target::of(params.container, params.project) {
            Ok(target) => target,
            Err(why) => return refuse(why),
        };
        let timeout =
            Duration::from_secs(params.timeout_seconds.unwrap_or(DEFAULT_WAIT).min(MAX_WAIT));
        let started = Instant::now();
        // A container is followed by ID, so a rename does not lose it.
        let mut id = None;
        loop {
            let containers = match self.containers().await {
                Ok((_, containers)) => containers,
                Err(why) => return refuse(why),
            };
            let found = match (&target, &id) {
                (_, Some(id)) => Ok(containers.iter().filter(|c| &c.id == id).collect()),
                (Target::Container(name), None) => {
                    find_container(&containers, name).map(|c| vec![c])
                }
                (Target::Project(name), None) => find_project(&containers, name),
            };
            let members: Vec<&Container> = match found {
                Ok(members) if !members.is_empty() => members,
                Ok(_) => return refuse("The container is gone."),
                Err(why) => return refuse(why),
            };
            if let (Target::Container(_), None) = (&target, &id) {
                id = Some(members[0].id.clone());
            }
            let state = readiness(&members);
            let waited = started.elapsed();
            let (ready, reason) = match state {
                Readiness::Ready => (true, None),
                Readiness::Stopped(why) => (false, Some(why)),
                Readiness::Waiting(why) if waited >= timeout => (false, Some(why)),
                Readiness::Waiting(_) => {
                    tokio::time::sleep(POLL).await;
                    continue;
                }
            };
            let report = WaitReport {
                target: match &target {
                    Target::Container(_) => members[0].display_name(),
                    Target::Project(name) => name.clone(),
                },
                ready,
                reason,
                waited_seconds: waited.as_secs(),
                containers: members
                    .iter()
                    .map(|c| format!("{}: {}", c.display_name(), status(c)))
                    .collect(),
            };
            return reply(&report, report.text());
        }
    }
}

/// The past lines of a stream that ends after them, or what arrived by `deadline`.
async fn read_past(
    mut lines: EngineStream<LogLine>,
    deadline: Instant,
) -> Result<Vec<LogLine>, String> {
    let mut read = Vec::new();
    loop {
        match tokio::time::timeout_at(deadline, lines.next()).await {
            Ok(Some(Ok(line))) => read.push(line),
            Ok(Some(Err(error))) => return Err(error.to_string()),
            Ok(None) | Err(_) => return Ok(read),
        }
    }
}

/// The Compose service, else the container's shown name.
fn source_name(container: &Container) -> String {
    container
        .compose
        .service
        .clone()
        .unwrap_or_else(|| container.display_name())
}

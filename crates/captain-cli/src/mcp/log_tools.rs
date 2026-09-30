//! `logs` and `wait_for_healthy`.

use std::time::Duration;

use captain_core::EngineStream;
use captain_core::agent_tools::{
    LogBuffer, LogQuery, LogReport, Readiness, SourcedLine, WaitReport, find_container,
    find_project, find_service, log_report, parse_since, readiness, status,
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
/// The least time the first look may take, so a timeout of 0 still looks once.
const FIRST_LOOK: Duration = Duration::from_secs(1);

#[tool_router(router = log_router, vis = "pub(super)")]
impl CaptainServer {
    /// The recent output of one container, or of a whole Compose project merged in
    /// time order. Defaults to the last 100 lines. Filters: service (with project),
    /// since, errors_only, grep. At most 500 lines or 32 KB come back, newest kept, with truncated and
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
        let (name, members, by_source) = match (&target, params.service.as_deref()) {
            (Target::Container(_), Some(_)) => {
                return refuse("Give service with project, not with container.");
            }
            (Target::Container(name), None) => match find_container(&containers, name) {
                Ok(container) => (container.display_name(), vec![container], false),
                Err(why) => return refuse(why),
            },
            (Target::Project(name), service) => {
                let found = find_project(&containers, name).and_then(|members| match service {
                    Some(service) => find_service(members, name, service),
                    None => Ok(members),
                });
                match (found, service) {
                    (Ok(members), Some(service)) => (format!("{name}/{service}"), members, true),
                    (Ok(members), None) => (name.clone(), members, true),
                    (Err(why), _) => return refuse(why),
                }
            }
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
            let query = &query;
            async move {
                let buffer = read_past(lines, query, deadline).await?;
                Ok::<_, String>((source, buffer))
            }
        });
        let mut lines = Vec::new();
        let mut matched = 0;
        for read in futures::future::join_all(reads).await {
            let (source, buffer) = match read {
                Ok(read) => read,
                Err(why) => return refuse(why),
            };
            matched += buffer.matched();
            lines.extend(buffer.into_lines().into_iter().map(|line| SourcedLine {
                source: source.clone(),
                line,
            }));
        }
        let report = log_report(&name, lines, matched, by_source);
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
        // One deadline covers connecting, each listing, and each sleep.
        let deadline = started + timeout;
        let mut target_name = match &target {
            Target::Container(name) | Target::Project(name) => name.clone(),
        };
        // A container is followed by ID, so a rename does not lose it.
        let mut id = None;
        let mut last = Vec::new();
        let mut answer_by = deadline.max(started + FIRST_LOOK);
        loop {
            let listed = tokio::time::timeout_at(answer_by, self.containers()).await;
            answer_by = deadline;
            let containers = match listed {
                Ok(Ok((_, containers))) => containers,
                Ok(Err(why)) => return refuse(why),
                Err(_) => {
                    let report = WaitReport {
                        target: target_name,
                        ready: false,
                        reason: Some(format!(
                            "The engine did not answer within {} s.",
                            started.elapsed().as_secs()
                        )),
                        waited_seconds: started.elapsed().as_secs(),
                        containers: last,
                    };
                    return reply(&report, report.text());
                }
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
                target_name = members[0].display_name();
            }
            last = members
                .iter()
                .map(|c| format!("{}: {}", c.display_name(), status(c)))
                .collect();
            let (ready, reason) = match readiness(&members) {
                Readiness::Ready => (true, None),
                Readiness::Stopped(why) => (false, Some(why)),
                Readiness::Waiting(why) if Instant::now() >= deadline => (false, Some(why)),
                Readiness::Waiting(_) => {
                    tokio::time::sleep_until(deadline.min(Instant::now() + POLL)).await;
                    continue;
                }
            };
            let report = WaitReport {
                target: target_name,
                ready,
                reason,
                waited_seconds: started.elapsed().as_secs(),
                containers: last,
            };
            return reply(&report, report.text());
        }
    }
}

/// The past lines of a stream that ends after them, or what arrived by `deadline`:
/// those that `query` keeps, within the buffer's limits as they arrive.
async fn read_past(
    mut lines: EngineStream<LogLine>,
    query: &LogQuery,
    deadline: Instant,
) -> Result<LogBuffer, String> {
    let mut buffer = LogBuffer::default();
    loop {
        match tokio::time::timeout_at(deadline, lines.next()).await {
            Ok(Some(Ok(line))) => buffer.push(query, line),
            Ok(Some(Err(error))) => return Err(error.to_string()),
            Ok(None) | Err(_) => return Ok(buffer),
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

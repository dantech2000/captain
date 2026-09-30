//! `container_problems`, `inspect`, and `disk_usage`.

use captain_core::agent_tools::{
    DiskReport, InspectReport, ProblemReport, disk_report, find_container, inspect_report,
    problem_report,
};
use rmcp::handler::server::tool::schema_for_output;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

use super::params::{ContainerParams, NoParams};
use super::reply::{ToolResult, refuse, reply};
use super::server::{CaptainServer, now};

#[tool_router(router = detail_router, vis = "pub(super)")]
impl CaptainServer {
    /// What needs attention, worst first, as Captain's menu bar sees it: crash
    /// loops, out-of-memory kills with the memory limit, exit codes, restart counts,
    /// failing health checks, and the fixes Captain offers. Also says when the
    /// engine does not run.
    #[tool(
        annotations(title = "Container problems", read_only_hint = true, open_world_hint = false),
        output_schema = schema_for_output::<ProblemReport>()
    )]
    async fn container_problems(&self, _: Parameters<NoParams>) -> ToolResult {
        let report = match self.containers().await {
            Ok((engine, containers)) => {
                let details = self.problem_details(&engine, &containers).await;
                problem_report(None, &containers, &details, &|id| self.source.crash(id))
            }
            Err(why) => problem_report(Some(why), &[], &Default::default(), &|_| None),
        };
        reply(&report, report.text())
    }

    /// One container's summary: command, ports, mounts, networks, limits, restart
    /// policy, last exit, and environment with secret-looking values masked. Never
    /// the raw inspect JSON.
    #[tool(
        annotations(title = "Inspect a container", read_only_hint = true, open_world_hint = false),
        output_schema = schema_for_output::<InspectReport>()
    )]
    async fn inspect(&self, Parameters(params): Parameters<ContainerParams>) -> ToolResult {
        let (engine, containers) = match self.containers().await {
            Ok(found) => found,
            Err(why) => return refuse(why),
        };
        let container = match find_container(&containers, &params.container) {
            Ok(container) => container,
            Err(why) => return refuse(why),
        };
        match engine.inspect_container(&container.id).await {
            Ok(detail) => {
                let report = inspect_report(container, &detail);
                reply(&report, report.text())
            }
            Err(error) => refuse(self.source.failed(error)),
        }
    }

    /// What the engine stores, as Captain's Storage page shows it: the categories,
    /// the largest items and who uses them, and what each cleanup group would free.
    /// It only reads; agents cannot clean up.
    #[tool(
        annotations(title = "Disk usage", read_only_hint = true, open_world_hint = false),
        output_schema = schema_for_output::<DiskReport>()
    )]
    async fn disk_usage(&self, _: Parameters<NoParams>) -> ToolResult {
        let engine = match self.source.engine().await {
            Ok(engine) => engine,
            Err(why) => return refuse(why),
        };
        let usage = match engine.disk_usage().await {
            Ok(usage) => usage,
            Err(error) => return refuse(self.source.failed(error)),
        };
        let host = self.source.host();
        let snapshots = match host.and_then(|host| host.snapshots()) {
            Some(snapshots) => snapshots.list().await.map(|list| list.snapshots).ok(),
            None => None,
        };
        let capacity = host.map(|host| host.resources().disk_bytes);
        let report = disk_report(&usage, snapshots.as_deref(), capacity, now());
        reply(&report, report.text())
    }
}

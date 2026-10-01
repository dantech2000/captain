//! `help`, `engine_status`, `list_projects`, and `list_containers`.

use captain_core::agent_tools::{
    ContainerList, EngineAnswer, EngineReport, HelpReport, ProjectList, container_list,
    engine_report, find_project, project_list,
};
use captain_core::problems::ExitFacts;
use rmcp::handler::server::tool::schema_for_output;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

use super::params::{ListContainersParams, NoParams};
use super::reply::{ToolResult, refuse, reply};
use super::server::CaptainServer;

#[tool_router(router = overview_router, vis = "pub(super)")]
impl CaptainServer {
    /// How Captain names things, what each tool does, and how container output is
    /// marked. Read it first.
    #[tool(
        annotations(title = "Help", read_only_hint = true, open_world_hint = false),
        output_schema = schema_for_output::<HelpReport>()
    )]
    async fn help(&self, _: Parameters<NoParams>) -> ToolResult {
        let report = self.help_report();
        reply(&report, report.text())
    }

    /// Which engine Captain uses and its state: running or why not, Docker version,
    /// CPUs, memory, disk, Kubernetes on or off, and how many containers run.
    #[tool(
        annotations(title = "Engine status", read_only_hint = true, open_world_hint = false),
        output_schema = schema_for_output::<EngineReport>()
    )]
    async fn engine_status(&self, _: Parameters<NoParams>) -> ToolResult {
        let host = self.source.host_status().await;
        let answer: EngineAnswer = match self.source.engine().await {
            Ok(engine) => futures::try_join!(engine.info(), engine.list_containers())
                .map(|(info, containers)| (info, self.shown(containers)))
                .map_err(|error| self.source.failed(error)),
            Err(why) => Err(why),
        };
        let report = engine_report(
            self.source.label,
            host.as_ref()
                .map(|(status, resources)| (status, *resources)),
            &answer,
            self.source.kubernetes,
        );
        reply(&report, report.text())
    }

    /// One row per Compose project, Kubernetes namespace, and the loose containers:
    /// containers running of total, services, health, published ports as URLs, the
    /// Compose folder, and the worst problem.
    #[tool(
        annotations(title = "List projects", read_only_hint = true, open_world_hint = false),
        output_schema = schema_for_output::<ProjectList>()
    )]
    async fn list_projects(&self, _: Parameters<NoParams>) -> ToolResult {
        let (engine, containers) = match self.containers().await {
            Ok(found) => found,
            Err(why) => return refuse(why),
        };
        let facts = self
            .problem_details(&engine, &containers)
            .await
            .iter()
            .map(|(id, detail)| (id.clone(), ExitFacts::of(detail)))
            .collect();
        let list = project_list(&containers, &facts, &|id| self.source.crash(id));
        reply(&list, list.text())
    }

    /// Every container as a short row: name, state, health, needs_attention, ID,
    /// project, service, image, published ports as URLs, and uptime. Pass
    /// status_only for the shortest form, or project for one Compose project.
    #[tool(
        annotations(title = "List containers", read_only_hint = true, open_world_hint = false),
        output_schema = schema_for_output::<ContainerList>()
    )]
    async fn list_containers(
        &self,
        Parameters(params): Parameters<ListContainersParams>,
    ) -> ToolResult {
        let mut containers = match self.containers().await {
            Ok((_, containers)) => containers,
            Err(why) => return refuse(why),
        };
        if let Some(project) = &params.project {
            let ids: Vec<String> = match find_project(&containers, project) {
                Ok(members) => members.iter().map(|c| c.id.clone()).collect(),
                Err(why) => return refuse(why),
            };
            containers.retain(|c| ids.contains(&c.id));
        }
        let crashed = |id: &str| self.source.crash(id).is_some();
        let list = container_list(&containers, params.status_only, &crashed);
        reply(&list, list.text())
    }
}

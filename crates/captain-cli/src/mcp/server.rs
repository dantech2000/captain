//! Captain's MCP server: the tools, and what it tells a client when it connects.
//! See docs/features/0038-agent-tools.md.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use captain_core::Engine;
use captain_core::agent_tools::{HelpReport, ToolLine};
use captain_core::model::{Container, ContainerDetail, ContainerState, Health};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::Tool;

use super::source::Source;

#[derive(Clone)]
pub struct CaptainServer {
    pub(super) source: Arc<Source>,
    pub(super) tool_router: ToolRouter<Self>,
}

impl CaptainServer {
    /// Every tool. [`Self::tools`] lists only those the settings allow now.
    pub fn new(source: Source) -> Self {
        Self {
            source: Arc::new(source),
            tool_router: Self::overview_router()
                + Self::detail_router()
                + Self::log_router()
                + Self::action_router(),
        }
    }

    /// The tools the `agent_tools` settings allow now, sorted by name.
    pub fn tools(&self) -> Vec<Tool> {
        let settings = self.source.agent_settings();
        self.tool_router
            .list_all()
            .into_iter()
            .filter(|tool| settings.gate(&tool.name).is_ok())
            .collect()
    }

    /// Every tool, allowed or not, for the reference page.
    pub fn all_tools(&self) -> Vec<Tool> {
        self.tool_router.list_all()
    }

    /// The `help` answer, with every tool this server lists now.
    pub(super) fn help_report(&self) -> HelpReport {
        let tools = self
            .tools()
            .into_iter()
            .map(|tool| ToolLine {
                name: tool.name.to_string(),
                description: tool.description.as_deref().unwrap_or_default().to_string(),
            })
            .collect();
        HelpReport::new(tools)
    }

    /// The engine and its containers, or why the engine does not answer.
    pub(super) async fn containers(&self) -> Result<(Arc<dyn Engine>, Vec<Container>), String> {
        let engine = self.source.engine().await?;
        let containers = engine
            .list_containers()
            .await
            .map_err(|error| self.source.failed(error))?;
        Ok((engine, containers))
    }

    /// `inspect` of each container that restarts, crashed lately, or fails its
    /// health check: the problem lines need its exit facts.
    pub(super) async fn problem_details(
        &self,
        engine: &Arc<dyn Engine>,
        containers: &[Container],
    ) -> HashMap<String, ContainerDetail> {
        let troubled = containers.iter().filter(|c| {
            c.state == ContainerState::Restarting
                || c.health == Some(Health::Unhealthy)
                || self.source.crash(&c.id).is_some()
        });
        let inspections = troubled.map(|c| {
            let inspect = engine.inspect_container(&c.id);
            async move { inspect.await.ok().map(|detail| (c.id.clone(), detail)) }
        });
        futures::future::join_all(inspections)
            .await
            .into_iter()
            .flatten()
            .collect()
    }
}

/// The current Unix time in seconds.
pub(super) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as i64)
}

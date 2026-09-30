//! The action tools: `start`, `stop`, `restart`, `run_task`, and `raise_memory`.
//! The server lists each one only while `agent_tools.actions` allows it, and refuses
//! a call to one that is off before it gets here (see `handler.rs`).

use captain_core::agent_tools::{
    ActionReport, AgentAction, TaskReport, check_name, find_container, find_project, memory_raise,
    raised_sentence, task_report,
};
use captain_core::model::{ContainerAction, ProjectAction};
use captain_core::store::compose_project;
use rmcp::handler::server::tool::schema_for_output;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

use super::params::{ActionParams, ContainerParams, Target, TaskParams};
use super::reply::{ToolResult, refuse, reply};
use super::server::CaptainServer;

#[tool_router(router = action_router, vis = "pub(super)")]
impl CaptainServer {
    /// Starts a stopped container, or a whole Compose project as `docker compose up
    /// -d` does. Give either container or project. Then call wait_for_healthy.
    #[tool(
        annotations(
            title = "Start",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        ),
        output_schema = schema_for_output::<ActionReport>()
    )]
    async fn start(&self, Parameters(params): Parameters<ActionParams>) -> ToolResult {
        answer(self.act(AgentAction::Start, params).await)
    }

    /// Stops a container, or every service of a Compose project. The containers
    /// stay, with their data. Give either container or project.
    #[tool(
        annotations(
            title = "Stop",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        ),
        output_schema = schema_for_output::<ActionReport>()
    )]
    async fn stop(&self, Parameters(params): Parameters<ActionParams>) -> ToolResult {
        answer(self.act(AgentAction::Stop, params).await)
    }

    /// Restarts a container, or every service of a Compose project. Give either
    /// container or project. Then call wait_for_healthy.
    #[tool(
        annotations(
            title = "Restart",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        ),
        output_schema = schema_for_output::<ActionReport>()
    )]
    async fn restart(&self, Parameters(params): Parameters<ActionParams>) -> ToolResult {
        answer(self.act(AgentAction::Restart, params).await)
    }

    /// Runs a task that the project's Compose file declares in x-captain.tasks, in
    /// its service, and returns its exit code and the end of its output. Only
    /// declared tasks run; list_projects and help do not list them, so ask the user
    /// or read the Compose file for the names.
    #[tool(
        annotations(
            title = "Run a task",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        ),
        output_schema = schema_for_output::<TaskReport>()
    )]
    async fn run_task(&self, Parameters(params): Parameters<TaskParams>) -> ToolResult {
        match self.task(params).await {
            Ok(report) => reply(&report, report.text()),
            Err(why) => refuse(why),
        }
    }

    /// Raises a container's memory limit the way Captain does after an
    /// out-of-memory kill: twice the old limit, at least 512 MB. The container
    /// keeps running. A container without a limit is refused.
    #[tool(
        annotations(
            title = "Raise memory",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        ),
        output_schema = schema_for_output::<ActionReport>()
    )]
    async fn raise_memory(&self, Parameters(params): Parameters<ContainerParams>) -> ToolResult {
        answer(self.raise(params).await)
    }
}

impl CaptainServer {
    async fn act(&self, action: AgentAction, params: ActionParams) -> Result<ActionReport, String> {
        let target = Target::of(params.container, params.project)?;
        let (engine, containers) = self.containers().await?;
        let (verb, past) = match action {
            AgentAction::Start => (ContainerAction::Start, "Started"),
            AgentAction::Stop => (ContainerAction::Stop, "Stopped"),
            _ => (ContainerAction::Restart, "Restarted"),
        };
        match target {
            Target::Container(name) => {
                let container = find_container(&containers, &name)?;
                let shown = container.display_name();
                engine
                    .run_action(&container.id, verb)
                    .await
                    .map_err(|error| self.source.failed(error))?;
                Ok(ActionReport::new(
                    action.name(),
                    &shown,
                    format!("{past} {shown}."),
                ))
            }
            Target::Project(name) => {
                find_project(&containers, &name)?;
                let project = compose_project(&containers, &name)
                    .ok_or_else(|| format!("No Compose project is named \"{name}\"."))?;
                let compose = match action {
                    AgentAction::Start => ProjectAction::Up,
                    AgentAction::Stop => ProjectAction::Stop,
                    _ => ProjectAction::Restart,
                };
                let runner = self.source.runner().await?;
                runner
                    .run_project(&project, compose)
                    .await
                    .map_err(|error| error.to_string())?;
                Ok(ActionReport::new(
                    action.name(),
                    &name,
                    format!("{past} the project {name}."),
                ))
            }
        }
    }

    async fn task(&self, params: TaskParams) -> Result<TaskReport, String> {
        let task_name = check_name("task", &params.task)?.to_string();
        let (_, containers) = self.containers().await?;
        find_project(&containers, &params.project)?;
        let project = compose_project(&containers, &params.project)
            .ok_or_else(|| format!("No Compose project is named \"{}\".", params.project))?;
        let runner = self.source.runner().await?;
        let tasks = runner
            .tasks(&project)
            .await
            .map_err(|error| error.to_string())?;
        let Some(task) = tasks.tasks.iter().find(|task| task.name == task_name) else {
            let names: Vec<&str> = tasks.tasks.iter().map(|task| task.name.as_str()).collect();
            return Err(format!(
                "The project {} has no task named \"{task_name}\". Its tasks: {}.",
                project.name,
                if names.is_empty() {
                    "none".to_string()
                } else {
                    names.join(", ")
                }
            ));
        };
        let output = runner
            .run_task(&project, task)
            .await
            .map_err(|error| error.to_string())?;
        Ok(task_report(&project.name, task, &output))
    }

    async fn raise(&self, params: ContainerParams) -> Result<ActionReport, String> {
        let (engine, containers) = self.containers().await?;
        let container = find_container(&containers, &params.container)?;
        let shown = container.display_name();
        let detail = engine
            .inspect_container(&container.id)
            .await
            .map_err(|error| self.source.failed(error))?;
        let raised = memory_raise(&shown, detail.memory_limit)?;
        engine
            .update_memory(&container.id, raised)
            .await
            .map_err(|error| self.source.failed(error))?;
        let message = raised_sentence(&shown, detail.memory_limit, raised);
        Ok(ActionReport::new(
            AgentAction::RaiseMemory.name(),
            &shown,
            message,
        ))
    }
}

fn answer(result: Result<ActionReport, String>) -> ToolResult {
    match result {
        Ok(report) => reply(&report, report.text()),
        Err(why) => refuse(why),
    }
}

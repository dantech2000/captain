//! The action tools behind the `agent_tools` settings, and the activity log.

use std::path::Path;
use std::sync::{Arc, Mutex};

use captain_core::agent_tools::{AgentAction, AgentToolsSettings, read_activity};
use captain_core::model::{
    ComposeProject, ProjectAction, ProjectTask, ProjectTasks, TaskCommand, TaskOutput,
};
use captain_core::project_files::{EditableFile, LineProblem, UpPreview};
use captain_core::{EngineFuture, ProjectRunner};
use futures::FutureExt;
use serde_json::json;

use super::super::Source;
use super::{call, connect, enabled, fake, read, serve, text};

/// A `docker compose` that declares one task, `migrate` in `api`.
struct FakeRunner;

impl ProjectRunner for FakeRunner {
    fn version(&self) -> &str {
        "v2.29.1"
    }

    fn run_project(&self, _: &ComposeProject, _: ProjectAction) -> EngineFuture<()> {
        futures::future::ready(Ok(())).boxed()
    }

    fn tasks(&self, _: &ComposeProject) -> EngineFuture<ProjectTasks> {
        let task = ProjectTask {
            name: "migrate".into(),
            service: "api".into(),
            command: TaskCommand::Shell("make migrate".into()),
        };
        futures::future::ready(Ok(ProjectTasks {
            tasks: vec![task],
            problems: Vec::new(),
        }))
        .boxed()
    }

    fn run_task(&self, _: &ComposeProject, _: &ProjectTask) -> EngineFuture<TaskOutput> {
        let output = TaskOutput {
            exit_code: 0,
            output: "migrated 3 tables\n".into(),
        };
        futures::future::ready(Ok(output)).boxed()
    }

    // The editor's methods; no MCP tool calls them.
    fn dockerfiles(&self, _: &ComposeProject) -> EngineFuture<Vec<EditableFile>> {
        unreachable!()
    }

    fn check_compose(
        &self,
        _: &ComposeProject,
        _: &Path,
        _: String,
    ) -> EngineFuture<Vec<LineProblem>> {
        unreachable!()
    }

    fn check_dockerfile(&self, _: &Path, _: String) -> EngineFuture<Vec<LineProblem>> {
        unreachable!()
    }

    fn preview_up(&self, _: &ComposeProject) -> EngineFuture<UpPreview> {
        unreachable!()
    }

    fn apply_up(&self, _: &ComposeProject, _: &[String]) -> EngineFuture<String> {
        unreachable!()
    }
}

#[tokio::test]
async fn with_agent_tools_off_only_help_is_listed_and_calls_name_the_setting() {
    let off = Arc::new(Mutex::new(AgentToolsSettings::default()));
    let source = Source::new("Other engine", None, connect(fake(), None), false);
    let client = serve(source.with_settings(read(off))).await;
    let tools = client.list_all_tools().await.unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "help");
    let refused = call(&client, "logs", json!({ "project": "shop" })).await;
    assert_eq!(refused.is_error, Some(true));
    assert!(text(&refused).contains("agent_tools.enabled"));
}

#[tokio::test]
async fn an_action_runs_only_while_allowed_and_every_call_is_logged() {
    let dir = std::env::temp_dir().join(format!("captain-mcp-activity-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    let log = dir.join("agent-activity.jsonl");
    let settings = enabled();
    let source = Source::new(
        "Other engine",
        None,
        connect(fake(), Some(Arc::new(FakeRunner))),
        false,
    )
    .with_settings(read(settings.clone()))
    .with_activity(log.clone());
    let client = serve(source).await;

    let refused = call(&client, "restart", json!({ "container": "shop-api-1" })).await;
    assert_eq!(refused.is_error, Some(true));
    assert!(text(&refused).contains("agent_tools.actions"));

    for action in AgentAction::ALL {
        settings.lock().unwrap().set_allowed(action, true);
    }
    let names: Vec<String> = client
        .list_all_tools()
        .await
        .unwrap()
        .iter()
        .filter(|tool| tool.annotations.as_ref().unwrap().read_only_hint == Some(false))
        .map(|tool| tool.name.to_string())
        .collect();
    assert_eq!(
        names,
        ["raise_memory", "restart", "run_task", "start", "stop"]
    );

    let restarted = call(&client, "restart", json!({ "container": "shop-api-1" })).await;
    assert_eq!(restarted.is_error, Some(false), "{}", text(&restarted));
    let stopped = call(&client, "stop", json!({ "project": "shop" })).await;
    assert_eq!(stopped.is_error, Some(false), "{}", text(&stopped));
    let task = call(
        &client,
        "run_task",
        json!({ "project": "shop", "task": "migrate" }),
    )
    .await;
    let report = task.structured_content.unwrap();
    assert_eq!(report["exit_code"], 0);
    assert!(
        report["output"]
            .as_str()
            .unwrap()
            .contains("migrated 3 tables")
    );
    let unknown = call(
        &client,
        "run_task",
        json!({ "project": "shop", "task": "seed" }),
    )
    .await;
    assert!(text(&unknown).contains("Its tasks: migrate."));
    // The fake engine's containers have no memory limit.
    let raise = call(
        &client,
        "raise_memory",
        json!({ "container": "shop-web-1" }),
    )
    .await;
    assert!(text(&raise).contains("no memory limit"));

    let logged = read_activity(&log, 10);
    let tools: Vec<&str> = logged.iter().map(|entry| entry.tool.as_str()).collect();
    assert_eq!(
        tools,
        [
            "raise_memory",
            "run_task",
            "run_task",
            "stop",
            "restart",
            "restart"
        ]
    );
    assert!(!logged[5].ok && logged[4].ok);
    assert_eq!(logged[4].summary(), "restart shop-api-1");
    std::fs::remove_dir_all(&dir).ok();
}

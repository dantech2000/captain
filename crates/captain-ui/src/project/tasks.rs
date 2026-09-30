use captain_core::EngineError;
use captain_core::model::{ComposeProject, ProjectTask, ProjectTasks, TaskOutput};
use gpui_kit::*;

use super::{ProjectNotice, ProjectView};

/// The tasks of the shown Compose project.
#[derive(Default)]
pub enum TaskList {
    /// Not a Compose project, or no `docker compose`.
    #[default]
    None,
    Loading,
    Ready(ProjectTasks),
    Failed(String),
}

/// The task list, the task that runs, and the end of the last run.
#[derive(Default)]
pub struct TaskState {
    pub list: TaskList,
    pub running: Option<String>,
    pub last: Option<(String, TaskOutput)>,
    load: Option<Task<()>>,
}

impl ProjectView {
    /// Reads `x-captain.tasks` of `project` again.
    pub(super) fn load_tasks(&mut self, project: &ComposeProject, cx: &mut Context<Self>) {
        let Some(runner) = self.workspace.read(cx).project_runner() else {
            self.tasks = TaskState::default();
            return;
        };
        self.tasks.list = TaskList::Loading;
        let read = runner.tasks(project);
        self.tasks.load = Some(cx.spawn(async move |this, cx| {
            let result = read.await;
            this.update(cx, |this, cx| {
                this.tasks.list = match result {
                    Ok(tasks) => TaskList::Ready(tasks),
                    Err(error) => TaskList::Failed(error.to_string()),
                };
                cx.notify();
            })
            .ok();
        }));
    }

    /// Runs `task` and keeps its output for the card. One task runs at a time.
    pub(super) fn run_task(&mut self, task: ProjectTask, cx: &mut Context<Self>) {
        let (Some(project), Some(runner)) = (
            self.project.clone(),
            self.workspace.read(cx).project_runner(),
        ) else {
            return;
        };
        if self.tasks.running.is_some() {
            return;
        }
        self.tasks.running = Some(task.name.clone());
        cx.notify();
        let run = runner.run_task(&project, &task);
        cx.spawn(async move |this, cx| {
            let result: Result<TaskOutput, EngineError> = run.await;
            this.update(cx, |this, cx| {
                this.tasks.running = None;
                match result {
                    Ok(output) => {
                        cx.emit(ProjectNotice::TaskDone {
                            task: task.name.clone(),
                            exit_code: output.exit_code,
                        });
                        this.tasks.last = Some((task.name, output));
                    }
                    Err(error) => cx.emit(ProjectNotice::Failed {
                        title: format!("{} did not run", task.name),
                        error: error.to_string(),
                    }),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

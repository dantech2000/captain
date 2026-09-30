use std::time::SystemTime;

use captain_core::EngineError;
use captain_core::model::{ComposeProject, ProjectTask, ProjectTasks, TaskOutput};
use captain_core::store::TaskRuns;
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

/// The shown project's task list, and the runs of every project. The runs stay when
/// the page shows another entry.
#[derive(Default)]
pub struct TaskState {
    pub list: TaskList,
    pub runs: TaskRuns,
    load: Option<Task<()>>,
    /// When the Compose files were last changed, as read for the list shown.
    stamp: Vec<Option<SystemTime>>,
}

impl TaskState {
    /// Forgets the task list for a new entry. Runs keep going.
    pub fn forget_list(&mut self) {
        self.list = TaskList::None;
        self.load = None;
        self.stamp.clear();
    }

    /// True if the list is not read yet, or a Compose file of `project` changed on
    /// disk since it was read, for example after the user added a task.
    pub fn stale(&self, project: &ComposeProject) -> bool {
        matches!(self.list, TaskList::None) || files_stamp(project) != self.stamp
    }
}

/// The modification time of each Compose file of `project`, or `None` for one that
/// cannot be read. A few `stat` calls, cheap enough to run on each refresh.
fn files_stamp(project: &ComposeProject) -> Vec<Option<SystemTime>> {
    project
        .config_files
        .iter()
        .map(|file| {
            std::fs::metadata(file)
                .and_then(|meta| meta.modified())
                .ok()
        })
        .collect()
}

impl ProjectView {
    /// Reads `x-captain.tasks` of `project` again.
    pub(super) fn load_tasks(&mut self, project: &ComposeProject, cx: &mut Context<Self>) {
        let Some(runner) = self.workspace.read(cx).project_runner() else {
            self.tasks.forget_list();
            return;
        };
        self.tasks.list = TaskList::Loading;
        self.tasks.stamp = files_stamp(project);
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

    /// Runs `task` and keeps its output for the card. Each project runs one task at
    /// a time.
    pub(super) fn run_task(&mut self, task: ProjectTask, cx: &mut Context<Self>) {
        let (Some(project), Some(runner)) = (
            self.project.clone(),
            self.workspace.read(cx).project_runner(),
        ) else {
            return;
        };
        let Some(id) = self.tasks.runs.start(&project.name, &task.name) else {
            return;
        };
        cx.notify();
        let run = runner.run_task(&project, &task);
        cx.spawn(async move |this, cx| {
            let result: Result<TaskOutput, EngineError> = run.await;
            this.update(cx, |this, cx| {
                let output = result.as_ref().ok().cloned();
                this.tasks.runs.finish(&project.name, id, output);
                match result {
                    Ok(output) => cx.emit(ProjectNotice::TaskDone {
                        task: task.name,
                        exit_code: output.exit_code,
                    }),
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

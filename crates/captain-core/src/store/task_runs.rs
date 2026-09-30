use std::collections::HashMap;

use crate::model::TaskOutput;

/// One project's tasks: the run in progress and the end of the last run.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ProjectRun {
    /// The run's number and the task's name.
    pub running: Option<(u64, String)>,
    /// The task's name and how it ended.
    pub last: Option<(String, TaskOutput)>,
}

/// Task runs by Compose project. Each project runs one task at a time, and a run
/// that ends updates only its own project, whatever the window shows by then.
#[derive(Debug, Default)]
pub struct TaskRuns {
    projects: HashMap<String, ProjectRun>,
    next: u64,
}

impl TaskRuns {
    pub fn get(&self, project: &str) -> Option<&ProjectRun> {
        self.projects.get(project)
    }

    /// Starts `task` in `project` and returns the run's number, or `None` while a
    /// task of `project` still runs.
    pub fn start(&mut self, project: &str, task: &str) -> Option<u64> {
        let run = self.projects.entry(project.to_string()).or_default();
        if run.running.is_some() {
            return None;
        }
        self.next += 1;
        run.running = Some((self.next, task.to_string()));
        Some(self.next)
    }

    /// Ends run `id` of `project`, and keeps `output` if the task ran. Returns false
    /// if that run is not the one in progress.
    pub fn finish(&mut self, project: &str, id: u64, output: Option<TaskOutput>) -> bool {
        let Some(run) = self.projects.get_mut(project) else {
            return false;
        };
        let Some((_, task)) = run.running.take_if(|(running, _)| *running == id) else {
            return false;
        };
        if let Some(output) = output {
            run.last = Some((task, output));
        }
        true
    }
}

#[cfg(test)]
mod tests;

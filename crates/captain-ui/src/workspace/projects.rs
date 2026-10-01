use std::sync::Arc;

use captain_core::ProjectRunner;
use captain_core::model::{ComposeProject, ProjectAction};
use captain_core::store::{ContainerGroup, compose_project, compose_projects};
use gpui_kit::*;

use super::{Workspace, WorkspaceEvent};

impl Workspace {
    /// Sets the Compose runner. `None` hides the Compose-only project actions.
    pub fn set_project_runner(
        &mut self,
        runner: Option<Arc<dyn ProjectRunner>>,
        cx: &mut Context<Self>,
    ) {
        if runner.is_none() {
            tracing::info!("docker compose is not available; project actions use the engine");
        }
        self.projects = runner;
        cx.notify();
    }

    /// True if `docker compose` actions can run.
    pub fn has_project_runner(&self) -> bool {
        self.projects.is_some()
    }

    /// Every Compose project, rebuilt from the container labels.
    pub fn compose_projects(&self) -> Vec<ComposeProject> {
        compose_projects(self.store.containers())
    }

    /// The Compose project `name`, from the container labels.
    pub fn compose_project(&self, name: &str) -> Option<ComposeProject> {
        compose_project(self.store.containers(), name)
    }

    /// Runs a `docker compose` command on the project `name`. The event stream
    /// reloads the list as containers change. It emits a [`WorkspaceEvent`] when the
    /// command ends.
    pub fn run_project_action(
        &mut self,
        name: String,
        action: ProjectAction,
        cx: &mut Context<Self>,
    ) {
        if let Some(project) = compose_project(self.store.containers(), &name) {
            self.run_project_action_on(project, action, cx);
        }
    }

    /// Like [`Self::run_project_action`], for a project the caller knows, for
    /// example one whose containers `down` removed.
    pub fn run_project_action_on(
        &mut self,
        project: ComposeProject,
        action: ProjectAction,
        cx: &mut Context<Self>,
    ) {
        let Some(runner) = self.projects.clone() else {
            return;
        };
        let name = project.name.clone();
        if self.project_pending.contains_key(&name) {
            return;
        }
        self.project_pending.insert(name.clone(), action);
        cx.notify();
        let run = runner.run_project(&project, action);
        cx.spawn(async move |this, cx| {
            let result = run.await;
            this.update(cx, |this, cx| {
                this.project_pending.remove(&name);
                match result {
                    Ok(()) => cx.emit(WorkspaceEvent::ProjectDone {
                        project: name,
                        action,
                    }),
                    Err(error) => {
                        tracing::warn!(%error, ?action, "project action failed");
                        cx.emit(WorkspaceEvent::ProjectFailed {
                            project: name,
                            action,
                            error,
                        });
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// The Compose command running on `project`, if any.
    pub fn project_pending(&self, project: &str) -> Option<ProjectAction> {
        self.project_pending.get(project).copied()
    }

    /// The project the Containers page shows alone, if any.
    pub fn project_filter(&self) -> Option<&str> {
        self.project_filter.as_deref()
    }

    /// Shows only `project` on the Containers page.
    pub fn set_project_filter(&mut self, project: String, cx: &mut Context<Self>) {
        self.project_filter = Some(project);
        cx.notify();
    }

    pub fn clear_project_filter(&mut self, cx: &mut Context<Self>) {
        self.project_filter = None;
        cx.notify();
    }

    /// The cards the Containers page shows: the state filter, then the project filter.
    pub fn visible_groups(&self) -> Vec<ContainerGroup> {
        let mut groups = self.store.groups(self.filter, self.show_kubernetes);
        if let Some(project) = &self.project_filter {
            groups.retain(|group| group.project() == Some(project.as_str()));
        }
        groups
    }
}

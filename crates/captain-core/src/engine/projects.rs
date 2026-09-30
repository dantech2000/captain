use std::path::Path;

use super::EngineFuture;
use crate::model::{ComposeProject, ProjectAction, ProjectTask, ProjectTasks, TaskOutput};
use crate::project_files::{EditableFile, LineProblem, UpPreview};

/// Runs Compose commands on whole projects. The Engine API has no Compose endpoints,
/// so this is separate from [`Engine`](crate::Engine): the Docker implementation
/// calls the `docker compose` CLI. See docs/adr/0005-compose-via-cli.md.
pub trait ProjectRunner: Send + Sync + 'static {
    /// The Compose version, for example `v2.29.1`.
    fn version(&self) -> &str;

    /// Runs `action` on `project`. The future ends when the command exits. It fails
    /// with the command's error output.
    fn run_project(&self, project: &ComposeProject, action: ProjectAction) -> EngineFuture<()>;

    /// The tasks in `x-captain.tasks` of the project's Compose files. It fails when
    /// Captain cannot find or read the files.
    fn tasks(&self, project: &ComposeProject) -> EngineFuture<ProjectTasks>;

    /// Runs `task` in its service, like `docker compose exec -T`. A task that exits
    /// with an error still gives its output; only a command that cannot start fails.
    fn run_task(&self, project: &ComposeProject, task: &ProjectTask) -> EngineFuture<TaskOutput>;

    /// The Dockerfile of each service that builds from a folder inside the project,
    /// read from `docker compose config`.
    fn dockerfiles(&self, project: &ComposeProject) -> EngineFuture<Vec<EditableFile>>;

    /// Checks the unsaved `text` of the Compose file `file` of `project` with
    /// `docker compose config --quiet`, the text on stdin in the file's place.
    fn check_compose(
        &self,
        project: &ComposeProject,
        file: &Path,
        text: String,
    ) -> EngineFuture<Vec<LineProblem>>;

    /// Checks an unsaved Dockerfile with BuildKit's build checks, with `context`
    /// as the build context. Nothing is built.
    fn check_dockerfile(&self, context: &Path, text: String) -> EngineFuture<Vec<LineProblem>>;

    /// What `up -d --remove-orphans` would do, from a dry run with a time limit.
    fn preview_up(&self, project: &ComposeProject) -> EngineFuture<UpPreview>;

    /// Runs `up -d --remove-orphans`, or with `build` services, `up -d --build`
    /// for those. The future gives Compose's output.
    fn apply_up(&self, project: &ComposeProject, build: &[String]) -> EngineFuture<String>;
}

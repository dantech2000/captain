use super::EngineFuture;
use crate::model::{ComposeProject, ProjectAction};

/// Runs Compose commands on whole projects. The Engine API has no Compose endpoints,
/// so this is separate from [`Engine`](crate::Engine): the Docker implementation
/// calls the `docker compose` CLI. See docs/adr/0005-compose-via-cli.md.
pub trait ProjectRunner: Send + Sync + 'static {
    /// The Compose version, for example `v2.29.1`.
    fn version(&self) -> &str;

    /// Runs `action` on `project`. The future ends when the command exits. It fails
    /// with the command's error output.
    fn run_project(&self, project: &ComposeProject, action: ProjectAction) -> EngineFuture<()>;
}

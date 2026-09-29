//! Runs every project action through the `docker compose` CLI against the real engine.
//! It writes its own project, `captain-agent-compose`, with one `busybox` service in a
//! temp folder, and runs `down` on it again even when an assertion fails.
//! Ignored by default: `cargo test -p captain-docker --test live_compose -- --ignored`.

use std::path::PathBuf;
use std::process::Command;

use captain_core::model::{ComposeProject, ProjectAction, ProjectStatus};
use captain_core::store::compose_project;
use captain_core::{ContainerApi, ProjectRunner};
use captain_docker::{ComposeCli, DiscoveryInput, DockerEngine, Endpoint, discover};
use futures::executor::block_on;

const NAME: &str = "captain-agent-compose";
const COMPOSE_FILE: &str = "\
services:
  box:
    image: busybox:latest
    command: [\"sleep\", \"300\"]
";

/// The temp folder with the Compose file. Dropping it runs `down` on the project and
/// removes the folder.
struct Fixture {
    dir: PathBuf,
    host: String,
}

impl Fixture {
    fn new(endpoint: &Endpoint) -> Self {
        let dir = std::env::temp_dir().join(NAME);
        std::fs::create_dir_all(&dir).expect("temp folder");
        std::fs::write(dir.join("compose.yaml"), COMPOSE_FILE).expect("compose file");
        let fixture = Self {
            dir,
            host: captain_docker_host(endpoint),
        };
        // A project left behind by an earlier, killed run.
        fixture.down();
        fixture
    }

    fn file(&self) -> PathBuf {
        self.dir.join("compose.yaml")
    }

    /// `docker compose -p captain-agent-compose -f <file> down`. Errors are ignored.
    fn down(&self) {
        Command::new("docker")
            .args(["compose", "-p", NAME, "-f"])
            .arg(self.file())
            .arg("down")
            .env("DOCKER_HOST", &self.host)
            .env_remove("DOCKER_CONTEXT")
            .output()
            .ok();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.down();
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

fn captain_docker_host(endpoint: &Endpoint) -> String {
    match endpoint.to_string().strip_prefix("http://") {
        Some(address) => format!("tcp://{address}"),
        None => endpoint.to_string(),
    }
}

/// The project as Captain sees it from the container labels.
fn project(engine: &DockerEngine) -> Option<ComposeProject> {
    let containers = block_on(engine.list_containers()).expect("list");
    compose_project(&containers, NAME)
}

fn run(cli: &ComposeCli, project: &ComposeProject, action: ProjectAction) {
    block_on(cli.run_project(project, action)).unwrap_or_else(|e| panic!("{action:?}: {e}"));
}

#[test]
#[ignore = "needs a running Docker engine and docker compose; creates and removes captain-agent-compose"]
fn up_stop_restart_pull_and_down_a_project() {
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    let fixture = Fixture::new(&endpoint);
    let cli = ComposeCli::detect(&endpoint).expect("docker compose");
    assert!(!cli.version().is_empty());
    let engine = DockerEngine::connect(endpoint).expect("connect");

    // Before the first `up` there are no labels yet, so build the project by hand.
    let fresh = ComposeProject {
        name: NAME.into(),
        working_dir: Some(fixture.dir.display().to_string()),
        config_files: vec![fixture.file().display().to_string()],
        services: Vec::new(),
    };
    run(&cli, &fresh, ProjectAction::Up);

    let up = project(&engine).expect("project after up");
    assert_eq!(up.status(), ProjectStatus::Running);
    assert_eq!(up.services.len(), 1);
    assert_eq!(up.services[0].name, "box");
    assert_eq!(up.config_files.len(), 1);
    assert!(up.working_dir.is_some());

    run(&cli, &up, ProjectAction::Stop);
    assert_eq!(
        project(&engine).map(|p| p.status()),
        Some(ProjectStatus::Stopped)
    );

    run(&cli, &up, ProjectAction::Restart);
    assert_eq!(
        project(&engine).map(|p| p.status()),
        Some(ProjectStatus::Running)
    );

    run(&cli, &up, ProjectAction::Pull);

    run(&cli, &up, ProjectAction::Down);
    assert_eq!(project(&engine), None);
}

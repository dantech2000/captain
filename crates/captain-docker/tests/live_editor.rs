//! Runs the project editor's commands against the real engine: the Dockerfile list,
//! both checks, the `up --dry-run` preview, and `up`. It writes its own project,
//! `captain-agent-editor`, in a temp folder, and removes it again even when an
//! assertion fails.
//! Ignored by default: `cargo test -p captain-docker --test live_editor -- --ignored`.

use std::path::PathBuf;
use std::process::Command;

use captain_core::model::ComposeProject;
use captain_core::project_files::{ChangeKind, Severity};
use captain_core::store::compose_project;
use captain_core::{ContainerApi, ProjectRunner};
use captain_docker::{ComposeCli, DiscoveryInput, DockerEngine, Endpoint, discover};
use futures::executor::block_on;

const NAME: &str = "captain-agent-editor";

fn compose(web_image: &str) -> String {
    format!(
        "services:\n  web:\n    image: {web_image}\n    command: [\"sleep\", \"300\"]\n  \
         api:\n    build: ./app\n    command: [\"sleep\", \"300\"]\n"
    )
}

/// The temp folder with the project. Dropping it runs `down --rmi local` and
/// removes the folder.
struct Fixture {
    dir: PathBuf,
    host: String,
}

impl Fixture {
    fn new(endpoint: &Endpoint) -> Self {
        let dir = std::env::temp_dir().join(NAME);
        std::fs::create_dir_all(dir.join("app")).expect("temp folder");
        std::fs::write(dir.join("compose.yaml"), compose("busybox:1.36")).expect("compose");
        std::fs::write(dir.join("app/Dockerfile"), "FROM busybox:1.36\n").expect("dockerfile");
        let host = match endpoint.to_string().strip_prefix("http://") {
            Some(address) => format!("tcp://{address}"),
            None => endpoint.to_string(),
        };
        let fixture = Self { dir, host };
        fixture.down();
        fixture
    }

    fn file(&self) -> PathBuf {
        self.dir.join("compose.yaml")
    }

    fn down(&self) {
        Command::new("docker")
            .args(["compose", "-p", NAME, "-f"])
            .arg(self.file())
            .args(["down", "--rmi", "local"])
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

/// The ID of each container, by service.
fn ids(engine: &DockerEngine) -> Vec<(String, String)> {
    let containers = block_on(engine.list_containers()).expect("list");
    let project = compose_project(&containers, NAME).expect("project");
    project
        .services
        .iter()
        .map(|s| (s.name.clone(), s.containers[0].id.clone()))
        .collect()
}

#[test]
#[ignore = "needs a running Docker engine with docker compose and buildx; creates and removes captain-agent-editor"]
fn checks_preview_and_apply_a_changed_project() {
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    let fixture = Fixture::new(&endpoint);
    let cli = ComposeCli::detect(&endpoint).expect("docker compose");
    let engine = DockerEngine::connect(endpoint).expect("connect");
    let fresh = ComposeProject {
        name: NAME.into(),
        working_dir: Some(fixture.dir.display().to_string()),
        config_files: vec!["compose.yaml".into()],
        services: Vec::new(),
    };
    block_on(cli.apply_up(&fresh, &[])).expect("first up");
    let containers = block_on(engine.list_containers()).expect("list");
    let project = compose_project(&containers, NAME).expect("project");

    let dockerfiles = block_on(cli.dockerfiles(&project)).expect("dockerfiles");
    assert_eq!(dockerfiles.len(), 1);
    assert_eq!(dockerfiles[0].services, ["api"]);

    let bad = compose("busybox:1.36").replace(
        "    command: [\"sleep\", \"300\"]\n  api",
        "    imgae: x\n  api",
    );
    let problems = block_on(cli.check_compose(&project, &fixture.file(), bad)).expect("check");
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].line, Some(3));

    let context = fixture.dir.join("app");
    let problems =
        block_on(cli.check_dockerfile(&context, "FROM busybox:1.36\nRUNN true\n".into()))
            .expect("build check");
    assert_eq!(problems[0].line, Some(1));
    assert_eq!(problems[0].severity, Severity::Error);

    let before = ids(&engine);
    std::fs::write(fixture.file(), compose("busybox:latest")).expect("edit");
    let preview = block_on(cli.preview_up(&project)).expect("preview");
    let changed: Vec<&str> = preview
        .changes
        .iter()
        .filter(|c| c.change != ChangeKind::Unchanged)
        .map(|c| c.service.as_str())
        .collect();
    assert_eq!(changed, ["web"]);

    block_on(cli.apply_up(&project, &[])).expect("apply");
    let after = ids(&engine);
    let recreated: Vec<&str> = before
        .iter()
        .filter(|(service, id)| after.iter().any(|(s, i)| s == service && i != id))
        .map(|(service, _)| service.as_str())
        .collect();
    assert_eq!(recreated, changed);
}

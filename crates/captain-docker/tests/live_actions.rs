//! Runs every container action against the real engine. It creates its own container,
//! `captain-agent-actions`, and removes it again even when an assertion fails.
//! Ignored by default: `cargo test -p captain-docker --test live_actions -- --ignored`.

use bollard::models::ContainerCreateBody;
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, RemoveContainerOptionsBuilder,
};
use bollard::{API_DEFAULT_VERSION, Docker};
use captain_core::ContainerApi;
use captain_core::model::{ContainerAction, ContainerState};
use captain_docker::{DiscoveryInput, DockerEngine, Endpoint, discover};
use futures::StreamExt;
use futures::executor::block_on;
use tokio::runtime::Runtime;

const NAME: &str = "captain-agent-actions";
const IMAGE: &str = "busybox:latest";

/// A raw client for the setup and cleanup that Captain's engine does not cover.
struct Fixture {
    runtime: Runtime,
    docker: Docker,
}

impl Fixture {
    fn new(endpoint: &Endpoint) -> Self {
        let docker = match endpoint {
            Endpoint::Tcp(address) => Docker::connect_with_http(address, 120, API_DEFAULT_VERSION),
            _ => Docker::connect_with_socket(&endpoint.to_string(), 120, API_DEFAULT_VERSION),
        }
        .expect("bollard client");
        Self {
            runtime: Runtime::new().expect("tokio runtime"),
            docker,
        }
    }

    /// Pulls the image if needed, then creates the test container. Returns its ID.
    fn create(&self) -> String {
        self.runtime.block_on(async {
            if self.docker.inspect_image(IMAGE).await.is_err() {
                let options = CreateImageOptionsBuilder::default()
                    .from_image(IMAGE)
                    .build();
                let mut pull = self.docker.create_image(Some(options), None, None);
                while let Some(progress) = pull.next().await {
                    progress.expect("pull busybox");
                }
            }
            let options = CreateContainerOptionsBuilder::default().name(NAME).build();
            let body = ContainerCreateBody {
                image: Some(IMAGE.into()),
                cmd: Some(vec!["sleep".into(), "300".into()]),
                ..Default::default()
            };
            let created = self.docker.create_container(Some(options), body).await;
            created.expect("create container").id
        })
    }

    /// Force-removes the test container. Missing is fine.
    fn remove(&self) {
        let options = RemoveContainerOptionsBuilder::default().force(true).build();
        self.runtime
            .block_on(self.docker.remove_container(NAME, Some(options)))
            .ok();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.remove();
    }
}

fn state(engine: &DockerEngine, id: &str) -> Option<ContainerState> {
    let containers = block_on(engine.list_containers()).expect("list");
    containers.into_iter().find(|c| c.id == id).map(|c| c.state)
}

fn run(engine: &DockerEngine, id: &str, action: ContainerAction) {
    block_on(engine.run_action(id, action)).unwrap_or_else(|e| panic!("{action:?}: {e}"));
}

#[test]
#[ignore = "needs a running Docker engine; creates and removes captain-agent-actions"]
fn pause_resume_stop_and_delete_a_container() {
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    let fixture = Fixture::new(&endpoint);
    // A container left behind by an earlier, killed run.
    fixture.remove();
    let id = fixture.create();
    let engine = DockerEngine::connect(endpoint).expect("connect");

    run(&engine, &id, ContainerAction::Start);
    assert_eq!(state(&engine, &id), Some(ContainerState::Running));

    run(&engine, &id, ContainerAction::Pause);
    assert_eq!(state(&engine, &id), Some(ContainerState::Paused));

    run(&engine, &id, ContainerAction::Unpause);
    assert_eq!(state(&engine, &id), Some(ContainerState::Running));

    let refused = block_on(engine.run_action(&id, ContainerAction::Remove));
    assert!(
        refused.is_err(),
        "a plain remove must refuse a running container"
    );

    run(&engine, &id, ContainerAction::Stop);
    assert_eq!(state(&engine, &id), Some(ContainerState::Exited));

    run(&engine, &id, ContainerAction::Start);
    run(&engine, &id, ContainerAction::ForceRemove);
    assert_eq!(state(&engine, &id), None);
}

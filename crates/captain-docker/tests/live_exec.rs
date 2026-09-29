//! Runs an interactive exec with a TTY in a real container: it writes a command, reads
//! the output back, resizes the TTY, and exits the shell. It creates its own container,
//! `captain-agent-exec`, and removes it again even when an assertion fails.
//! Ignored by default: `cargo test -p captain-docker --test live_exec -- --ignored`.

use std::time::Duration;

use bollard::models::ContainerCreateBody;
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, RemoveContainerOptionsBuilder,
};
use bollard::{API_DEFAULT_VERSION, Docker};
use captain_core::ContainerApi;
use captain_core::EngineStream;
use captain_core::model::ExecSpec;
use captain_docker::{DiscoveryInput, DockerEngine, Endpoint, discover};
use futures::StreamExt;
use tokio::runtime::Runtime;

const NAME: &str = "captain-agent-exec";
const IMAGE: &str = "busybox:latest";
const WAIT: Duration = Duration::from_secs(15);

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

    /// Pulls the image if needed, then starts `sleep 300`. Returns the ID.
    fn start_sleeper(&self) -> String {
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
            let id = self
                .docker
                .create_container(Some(options), body)
                .await
                .expect("create container")
                .id;
            self.docker
                .start_container(&id, None)
                .await
                .expect("start container");
            id
        })
    }

    /// Reads output until it contains `needle`, and returns all of it.
    fn read_until(&self, output: &mut EngineStream<Vec<u8>>, needle: &str) -> String {
        let mut text = String::new();
        self.runtime.block_on(async {
            tokio::time::timeout(WAIT, async {
                while !text.contains(needle) {
                    let chunk = output.next().await.expect("output ended").expect("chunk");
                    text.push_str(&String::from_utf8_lossy(&chunk));
                }
            })
            .await
            .unwrap_or_else(|_| panic!("no {needle:?} in {text:?}"));
        });
        text
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

#[test]
#[ignore = "needs a running Docker engine; creates and removes captain-agent-exec"]
fn runs_an_interactive_shell() {
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    let fixture = Fixture::new(&endpoint);
    // A container left behind by an earlier, killed run.
    fixture.remove();
    let id = fixture.start_sleeper();

    let engine = DockerEngine::connect(endpoint).expect("connect");
    let session = fixture
        .runtime
        .block_on(engine.exec(&id, ExecSpec::shell(80, 24)))
        .expect("exec");
    // busybox has no bash, so the probe falls back to sh.
    assert_eq!(session.command, ["/bin/sh"]);
    let mut output = session.output;

    session
        .input
        .send(b"echo hel''lo\n".to_vec())
        .expect("send");
    let text = fixture.read_until(&mut output, "hello");
    assert!(text.contains("hello\r\n"), "{text:?}");

    fixture
        .runtime
        .block_on(session.resizer.resize(100, 30))
        .expect("resize");
    session
        .input
        .send(b"stty size; echo $TERM\n".to_vec())
        .expect("send");
    // `stty size` prints rows, then columns, before the TERM line.
    let text = fixture.read_until(&mut output, "xterm-256color");
    assert!(text.contains("30 100"), "{text:?}");

    session.input.send(b"exit 3\n".to_vec()).expect("send");
    let code = fixture.runtime.block_on(async {
        tokio::time::timeout(WAIT, async {
            while output.next().await.is_some() {}
            session.exit.await
        })
        .await
        .expect("the shell exits")
    });
    assert_eq!(code, Ok(Some(3)));
}

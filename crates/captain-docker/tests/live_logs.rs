//! Reads the logs of a real container, with and without Docker timestamps. It creates
//! its own container, `captain-agent-logs`, and removes it again even when an
//! assertion fails.
//! Ignored by default: `cargo test -p captain-docker --test live_logs -- --ignored`.

use std::time::{SystemTime, UNIX_EPOCH};

use bollard::container::LogOutput;
use bollard::models::ContainerCreateBody;
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, LogsOptionsBuilder,
    RemoveContainerOptionsBuilder,
};
use bollard::{API_DEFAULT_VERSION, Docker};
use captain_core::ContainerApi;
use captain_core::model::LogStream;
use captain_docker::{DiscoveryInput, DockerEngine, Endpoint, discover};
use futures::StreamExt;
use futures::executor::block_on;
use tokio::runtime::Runtime;

const NAME: &str = "captain-agent-logs";
const IMAGE: &str = "busybox:latest";
const SCRIPT: &str = "echo alpha; echo bravo >&2; echo; echo charlie";

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

    /// Pulls the image if needed, then runs the script until it exits. Returns the ID.
    fn run_to_exit(&self) -> String {
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
                cmd: Some(vec!["sh".into(), "-c".into(), SCRIPT.into()]),
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
            let mut wait = self.docker.wait_container(&id, None);
            while let Some(status) = wait.next().await {
                status.expect("wait for exit");
            }
            id
        })
    }

    /// The container output without timestamps, straight from bollard.
    fn plain_logs(&self, id: &str) -> Vec<String> {
        self.runtime.block_on(async {
            let options = LogsOptionsBuilder::default()
                .stdout(true)
                .stderr(true)
                .build();
            let frames: Vec<_> = self.docker.logs(id, Some(options)).collect().await;
            frames
                .into_iter()
                .map(|frame| frame.expect("log frame"))
                .flat_map(|frame| {
                    let bytes = match frame {
                        LogOutput::StdOut { message }
                        | LogOutput::StdErr { message }
                        | LogOutput::Console { message }
                        | LogOutput::StdIn { message } => message,
                    };
                    let text = String::from_utf8_lossy(&bytes).into_owned();
                    text.lines().map(str::to_string).collect::<Vec<_>>()
                })
                .collect()
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

#[test]
#[ignore = "needs a running Docker engine; creates and removes captain-agent-logs"]
fn reads_logs_with_and_without_timestamps() {
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    let fixture = Fixture::new(&endpoint);
    // A container left behind by an earlier, killed run.
    fixture.remove();
    let before = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_secs() as i64;
    let id = fixture.run_to_exit();

    // Docker reads stdout and stderr separately, so only the order within a stream
    // is certain.
    let mut plain = fixture.plain_logs(&id);
    plain.sort();
    assert_eq!(plain, ["", "alpha", "bravo", "charlie"]);

    let engine = DockerEngine::connect(endpoint).expect("connect");
    let lines: Vec<_> = block_on(engine.logs(&id, 100).collect::<Vec<_>>())
        .into_iter()
        .map(|line| line.expect("log line"))
        .collect();
    let stream_texts = |stream: LogStream| -> Vec<&str> {
        lines
            .iter()
            .filter(|line| line.stream == stream)
            .map(|line| line.text.as_str())
            .collect()
    };
    // The blank line has no text after its timestamp, so the engine drops it.
    assert_eq!(stream_texts(LogStream::Stdout), ["alpha", "charlie"]);
    assert_eq!(stream_texts(LogStream::Stderr), ["bravo"]);
    for line in &lines {
        let time = line.timestamp.expect("a Docker timestamp");
        // The engine clock can differ a little from this machine's clock.
        assert!((time - before).abs() < 300, "{time} is far from {before}");
    }
}

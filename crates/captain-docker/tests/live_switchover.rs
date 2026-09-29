//! Runs the Migration Assistant's switch-over against the real engine that discovery
//! finds, with that one engine as both the source and the target. It touches only
//! what it creates, all named `captain-agent-sw*`, and removes it again even when an
//! assertion fails. Ignored by default:
//! `cargo test -p captain-docker --test live_switchover -- --ignored`.

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

use bollard::models::{ContainerCreateBody, HostConfig, PortBinding};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, ListContainersOptionsBuilder, LogsOptionsBuilder,
    RemoveContainerOptionsBuilder, RemoveVolumeOptions,
};
use bollard::{API_DEFAULT_VERSION, Docker};
use captain_core::migration::{MigrationItem, MigrationSession, SwitchOverStep, TransferEvent};
use captain_docker::{DiscoveryInput, DockerSession, Endpoint, discover};
use futures::executor::block_on;
use futures::{StreamExt, TryStreamExt};
use tokio::runtime::Runtime;

const SOURCE: &str = "captain-agent-sw-src";
const COPY: &str = "captain-agent-sw-dst";
const VOLUME: &str = "captain-agent-sw-vol";
const VOLUME_COPY: &str = "captain-agent-sw-copy";
const READER: &str = "captain-agent-sw-read";
const PROJECT: &str = "captain-agent-swc";
const IMAGE: &str = "busybox:latest";
/// Host ports the tests publish. Chosen to be unlikely to be in use.
const PORT: u16 = 18474;
const PROJECT_PORT: u16 = 18475;
const COMPOSE_FILE: &str = "\
services:
  web:
    image: busybox:latest
    init: true
    command: [\"sh\", \"-c\", \"mkdir -p /www && echo ok > /www/index.html && exec httpd -f -p 80 -h /www\"]
    ports: [\"18475:80\"]
    healthcheck:
      test: [\"CMD\", \"wget\", \"-q\", \"-O-\", \"http://127.0.0.1/\"]
      interval: 1s
  extra:
    image: busybox:latest
    profiles: [\"extra\"]
    command: [\"sleep\", \"300\"]
";

/// The tests share one engine, so they run one at a time.
static SERIAL: Mutex<()> = Mutex::new(());

fn endpoint() -> Endpoint {
    discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover")
}

fn listening(port: u16) -> bool {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    TcpStream::connect_timeout(&address, Duration::from_millis(500)).is_ok()
}

/// Bollard on its own runtime, for setup and checks.
struct Fixture {
    runtime: Runtime,
    docker: Docker,
    dir: PathBuf,
    host: String,
}

impl Fixture {
    fn new() -> Self {
        let endpoint = endpoint();
        let docker = match &endpoint {
            Endpoint::Tcp(address) => Docker::connect_with_http(address, 120, API_DEFAULT_VERSION),
            other => Docker::connect_with_socket(&other.to_string(), 120, API_DEFAULT_VERSION),
        }
        .expect("bollard");
        let dir = std::env::temp_dir().join(PROJECT);
        std::fs::create_dir_all(&dir).expect("temp folder");
        std::fs::write(dir.join("compose.yaml"), COMPOSE_FILE).expect("compose file");
        let host = endpoint.to_string().replace("http://", "tcp://");
        let runtime = Runtime::new().expect("runtime");
        let fixture = Self {
            runtime,
            docker,
            dir,
            host,
        };
        fixture.clean();
        fixture
    }

    fn compose(&self, args: &[&str]) -> bool {
        let output = Command::new("docker")
            .args(["compose", "-p", PROJECT, "-f"])
            .arg(self.dir.join("compose.yaml"))
            .args(args)
            .env("DOCKER_HOST", &self.host)
            .env_remove("DOCKER_CONTEXT")
            .output()
            .expect("docker compose");
        output.status.success()
    }

    /// Creates a container from busybox and starts it unless `start` is false.
    fn container(&self, name: &str, script: &str, host_config: HostConfig, start: bool) {
        let body = ContainerCreateBody {
            image: Some(IMAGE.into()),
            cmd: Some(vec!["sh".into(), "-c".into(), script.into()]),
            host_config: Some(host_config),
            ..ContainerCreateBody::default()
        };
        let options = CreateContainerOptionsBuilder::default().name(name).build();
        let docker = self.docker.clone();
        self.block(async move {
            docker.create_container(Some(options), body).await?;
            if start {
                docker.start_container(name, None).await?;
            }
            Ok::<_, bollard::errors::Error>(())
        })
        .expect("container");
    }

    /// Runs `script` with `volume` at `/v` and returns its output.
    fn read(&self, volume: &str, script: &str) -> String {
        let binds = Some(vec![format!("{volume}:/v")]);
        let host_config = HostConfig {
            binds,
            ..HostConfig::default()
        };
        self.container(READER, script, host_config, true);
        let docker = self.docker.clone();
        let output = self.block(async move {
            let _ = docker
                .wait_container(READER, None)
                .collect::<Vec<_>>()
                .await;
            let logs = LogsOptionsBuilder::default().stdout(true).build();
            let lines: Vec<_> = docker.logs(READER, Some(logs)).try_collect().await?;
            let force = RemoveContainerOptionsBuilder::default().force(true).build();
            docker.remove_container(READER, Some(force)).await.ok();
            Ok::<_, bollard::errors::Error>(lines)
        });
        let lines = output.expect("read");
        lines.into_iter().map(|line| line.to_string()).collect()
    }

    fn running(&self, name: &str) -> Option<bool> {
        let inspect = self.block(self.docker.inspect_container(name, None)).ok()?;
        inspect.state.and_then(|state| state.running)
    }

    fn project_services(&self) -> Vec<String> {
        let label = format!("com.docker.compose.project={PROJECT}");
        let filters = HashMap::from([("label", vec![label.as_str()])]);
        let options = ListContainersOptionsBuilder::default()
            .all(true)
            .filters(&filters)
            .build();
        let list = self.block(self.docker.list_containers(Some(options)));
        list.expect("list")
            .into_iter()
            .filter_map(|c| c.labels?.get("com.docker.compose.service").cloned())
            .collect()
    }

    fn block<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    fn clean(&self) {
        self.compose(&["--profile", "extra", "down", "--timeout", "1"]);
        let docker = self.docker.clone();
        self.block(async move {
            let force = RemoveContainerOptionsBuilder::default().force(true).build();
            for name in [SOURCE, COPY, READER] {
                docker
                    .remove_container(name, Some(force.clone()))
                    .await
                    .ok();
            }
            for name in [VOLUME, VOLUME_COPY] {
                let options = RemoveVolumeOptions { force: true };
                docker.remove_volume(name, Some(options)).await.ok();
            }
        });
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.clean();
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

fn session() -> DockerSession {
    let endpoint = endpoint();
    DockerSession::connect(&endpoint, &endpoint).expect("session")
}

/// The switch-over steps the events report, and whether they report a downtime.
fn steps(events: &[TransferEvent]) -> (Vec<SwitchOverStep>, bool) {
    let steps = events.iter().filter_map(|event| match event {
        TransferEvent::SwitchOver(step) => Some(*step),
        _ => None,
    });
    let downtime = events
        .iter()
        .any(|e| matches!(e, TransferEvent::Downtime(_)));
    (steps.collect(), downtime)
}

#[test]
#[ignore = "needs a running Docker engine"]
fn switches_a_container_over_and_rolls_back() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    assert!(!listening(PORT), "port {PORT} is in use; pick another");
    let fixture = Fixture::new();
    // A stale copy from an earlier run. The switch-over must replace it. A bind of
    // a named volume that does not exist creates it.
    fixture.read(VOLUME_COPY, "echo old > /v/stale");
    let port = PortBinding {
        host_ip: Some("127.0.0.1".into()),
        host_port: Some(PORT.to_string()),
    };
    let host_config = HostConfig {
        binds: Some(vec![format!("{VOLUME}:/data")]),
        port_bindings: Some(HashMap::from([("80/tcp".into(), Some(vec![port]))])),
        init: Some(true),
        ..HostConfig::default()
    };
    let script = "echo hello > /data/marker && mkdir -p /www && echo ok > /www/index.html \
                  && exec httpd -f -p 80 -h /www";
    fixture.container(SOURCE, script, host_config, true);

    let session = session();
    let volumes = [(VOLUME.to_string(), VOLUME_COPY.to_string())];
    let events: Vec<TransferEvent> = block_on(
        session
            .switch_container_as(SOURCE, COPY, &volumes)
            .try_collect(),
    )
    .expect("switch over");
    assert_eq!(steps(&events), (SwitchOverStep::SEQUENCE.to_vec(), true));
    // The original is stopped, not removed. The copy runs and answers.
    assert_eq!(fixture.running(SOURCE), Some(false));
    assert_eq!(fixture.running(COPY), Some(true));
    assert!(listening(PORT));
    let copy = fixture.read(VOLUME_COPY, "cat /v/marker; ls /v");
    assert_eq!(
        copy.split_whitespace().collect::<Vec<_>>(),
        ["hello", "marker"]
    );

    block_on(session.roll_back_container_as(SOURCE, COPY)).expect("roll back");
    assert_eq!(fixture.running(COPY), Some(false));
    assert_eq!(fixture.running(SOURCE), Some(true));
    block_on(session.finish()).expect("finish");
}

#[test]
#[ignore = "needs a running Docker engine and the docker compose CLI"]
fn switches_a_project_over_without_its_profile_services() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    assert!(!listening(PROJECT_PORT), "port {PROJECT_PORT} is in use");
    let fixture = Fixture::new();
    assert!(fixture.compose(&["up", "-d", "--wait"]), "compose up");
    let session = session();
    let plan = block_on(session.scan()).expect("scan");
    let item = plan
        .entries
        .into_iter()
        .map(|entry| entry.item)
        .find(|item| matches!(item, MigrationItem::ComposeProject { name, .. } if name == PROJECT))
        .expect("the project");
    assert!(item.can_switch_over());

    // One engine is both sides, so the project's own containers stop and start.
    let events: Vec<TransferEvent> =
        block_on(session.switch_over(&item, false).try_collect()).expect("switch over");
    assert_eq!(steps(&events), (SwitchOverStep::SEQUENCE.to_vec(), true));
    assert_eq!(fixture.project_services(), ["web"]);
    assert!(listening(PROJECT_PORT));

    block_on(session.roll_back(&item)).expect("roll back");
    assert_eq!(fixture.project_services(), ["web"]);
    block_on(session.finish()).expect("finish");
}

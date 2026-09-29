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

use bollard::models::{ContainerCreateBody, HostConfig, PortBinding, VolumeCreateRequest};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, ListContainersOptionsBuilder, LogsOptionsBuilder,
    RemoveContainerOptionsBuilder, RemoveVolumeOptions,
};
use bollard::{API_DEFAULT_VERSION, Docker};
use captain_core::migration::{MigrationItem, MigrationSession, SwitchOverStep, TransferEvent};
use captain_core::model::ComposeProject;
use captain_docker::{ComposeCli, DiscoveryInput, DockerSession, Endpoint, discover};
use futures::executor::block_on;
use futures::{StreamExt, TryStreamExt};
use tokio::runtime::Runtime;

const SOURCE: &str = "captain-agent-sw-src";
const COPY: &str = "captain-agent-sw-dst";
const VOLUME: &str = "captain-agent-sw-vol";
const VOLUME_COPY: &str = "captain-agent-sw-copy";
const READER: &str = "captain-agent-sw-read";
const OTHER: &str = "captain-agent-sw-other";
const PROJECT: &str = "captain-agent-swc";
const IMAGE: &str = "busybox:latest";
/// The label Captain puts on what it copied into the target.
const MIGRATED_FROM: &str = "dev.captain.migrated-from";
const HTTPD: &str = "echo hello > /data/marker && mkdir -p /www && echo ok > /www/index.html \
                     && exec httpd -f -p 80 -h /www";
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
        // The project runs with --project-directory, away from its file.
        std::fs::create_dir_all(dir.join("app")).expect("project folder");
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
            .args(["compose", "-p", PROJECT, "--project-directory"])
            .arg(self.dir.join("app"))
            .arg("-f")
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

    /// Creates `name` as a volume that Captain copied from this engine, or else as
    /// one that was already in the target.
    fn volume(&self, name: &str, copied: bool) {
        let docker = self.docker.clone();
        let name = name.to_string();
        self.block(async move {
            let id = docker.info().await?.id.unwrap_or_default();
            let labels = copied.then(|| HashMap::from([(MIGRATED_FROM.to_string(), id)]));
            let request = VolumeCreateRequest {
                name: Some(name),
                labels,
                ..VolumeCreateRequest::default()
            };
            docker.create_volume(request).await.map(|_| ())
        })
        .expect("volume");
    }

    /// `compose up` the way a Captain copy does: every container gets the label.
    fn up_labeled(&self, item: &MigrationItem) {
        let MigrationItem::ComposeProject {
            working_dir,
            config_files,
            ..
        } = item
        else {
            panic!("not a project: {item:?}");
        };
        let cli = ComposeCli::detect(&endpoint()).expect("compose CLI");
        let project = ComposeProject {
            name: PROJECT.into(),
            working_dir: working_dir.clone(),
            config_files: config_files.clone(),
            services: Vec::new(),
        };
        let id = self.block(self.docker.info()).expect("info").id;
        let labels = HashMap::from([(MIGRATED_FROM.to_string(), id.unwrap_or_default())]);
        block_on(cli.up_labeled(&project, &[], labels)).expect("labeled up");
        assert!(
            self.labels(&format!("{PROJECT}-web-1"))
                .contains_key(MIGRATED_FROM)
        );
    }

    fn labels(&self, name: &str) -> HashMap<String, String> {
        let inspect = self.block(self.docker.inspect_container(name, None));
        let config = inspect.expect("inspect").config;
        config.and_then(|c| c.labels).unwrap_or_default()
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
            for name in [SOURCE, COPY, READER, OTHER] {
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
    // A stale copy from an earlier run. The switch-over must replace it.
    fixture.volume(VOLUME_COPY, true);
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
    fixture.container(SOURCE, HTTPD, host_config, true);

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
#[ignore = "needs a running Docker engine"]
fn refuses_to_empty_a_volume_that_captain_did_not_copy() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let fixture = Fixture::new();
    fixture.volume(VOLUME_COPY, false);
    fixture.read(VOLUME_COPY, "echo mine > /v/keep");
    let host_config = HostConfig {
        binds: Some(vec![format!("{VOLUME}:/data")]),
        init: Some(true),
        ..HostConfig::default()
    };
    fixture.container(SOURCE, HTTPD, host_config, true);

    let session = session();
    let volumes = [(VOLUME.to_string(), VOLUME_COPY.to_string())];
    let switched: Result<Vec<TransferEvent>, _> = block_on(
        session
            .switch_container_as(SOURCE, COPY, &volumes)
            .try_collect(),
    );
    let error = switched.expect_err("refused").to_string();
    assert!(error.contains("Captain did not copy it"), "{error}");
    // Nothing stopped, and the target volume keeps its data.
    assert_eq!(fixture.running(SOURCE), Some(true));
    assert_eq!(fixture.read(VOLUME_COPY, "cat /v/keep").trim(), "mine");
    block_on(session.finish()).expect("finish");
}

#[test]
#[ignore = "needs a running Docker engine"]
fn rolls_back_when_the_copy_was_never_created() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let fixture = Fixture::new();
    // As if the switch-over stopped the source and failed before the start.
    fixture.container(SOURCE, "exec sleep 300", HostConfig::default(), false);

    let session = session();
    block_on(session.roll_back_container_as(SOURCE, COPY)).expect("roll back");
    assert_eq!(fixture.running(SOURCE), Some(true));
    assert_eq!(fixture.running(COPY), None);
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

    // One engine is both sides. Its containers came from `compose up`, not from
    // Captain, so the switch-over refuses them and stops nothing.
    let refused: Result<Vec<TransferEvent>, _> =
        block_on(session.switch_over(&item, false).try_collect());
    let error = refused.expect_err("refused").to_string();
    assert!(error.contains("Captain did not copy it"), "{error}");
    assert_eq!(fixture.running(&format!("{PROJECT}-web-1")), Some(true));
    // An ordinary copy skips the project too, and does not relabel it.
    let events: Vec<TransferEvent> =
        block_on(session.copy(&item, false).try_collect()).expect("copy");
    assert!(matches!(events.last(), Some(TransferEvent::Skipped(_))));
    assert!(
        !fixture
            .labels(&format!("{PROJECT}-web-1"))
            .contains_key(MIGRATED_FROM)
    );

    // Recreated with Captain's label, as a Compose copy does, it switches over.
    // Its copy keeps the --project-directory folder, and Captain can replay it.
    fixture.up_labeled(&item);
    let rescanned = block_on(session.scan()).expect("scan");
    assert!(rescanned.entries.iter().any(|entry| matches!(
        &entry.item,
        MigrationItem::ComposeProject { name, files_exist: true, .. } if name == PROJECT
    )));
    let events: Vec<TransferEvent> =
        block_on(session.switch_over(&item, false).try_collect()).expect("switch over");
    assert_eq!(steps(&events), (SwitchOverStep::SEQUENCE.to_vec(), true));
    assert_eq!(fixture.project_services(), ["web"]);
    assert!(listening(PROJECT_PORT));

    block_on(session.roll_back(&item)).expect("roll back");
    assert_eq!(fixture.project_services(), ["web"]);
    block_on(session.finish()).expect("finish");
}

/// A Unix socket that forwards to the engine until [`Proxy::cut`], so a session can
/// connect through it and then lose the engine.
#[cfg(unix)]
struct Proxy {
    path: PathBuf,
    streams: std::sync::Arc<Mutex<Vec<std::os::unix::net::UnixStream>>>,
}

#[cfg(unix)]
impl Proxy {
    fn new(to: PathBuf) -> Self {
        use std::os::unix::net::{UnixListener, UnixStream};
        let path = std::env::temp_dir().join("captain-agent-sw-proxy.sock");
        std::fs::remove_file(&path).ok();
        let listener = UnixListener::bind(&path).expect("proxy socket");
        let streams = std::sync::Arc::new(Mutex::new(Vec::<UnixStream>::new()));
        let kept = streams.clone();
        std::thread::spawn(move || {
            for client in listener.incoming().flatten() {
                let Ok(server) = UnixStream::connect(&to) else {
                    break;
                };
                let pipe = |mut from: UnixStream, mut to: UnixStream| {
                    std::thread::spawn(move || std::io::copy(&mut from, &mut to).ok());
                };
                let mut list = kept.lock().unwrap_or_else(|e| e.into_inner());
                list.extend([client.try_clone().unwrap(), server.try_clone().unwrap()]);
                pipe(client.try_clone().unwrap(), server.try_clone().unwrap());
                pipe(server, client);
            }
        });
        Self { path, streams }
    }

    /// Closes every connection and removes the socket, so the engine is gone.
    fn cut(&self) {
        std::fs::remove_file(&self.path).ok();
        let streams = self.streams.lock().unwrap_or_else(|e| e.into_inner());
        for stream in streams.iter() {
            stream.shutdown(std::net::Shutdown::Both).ok();
        }
    }
}

#[cfg(unix)]
#[test]
#[ignore = "needs a running Docker engine on a Unix socket"]
fn roll_back_leaves_the_source_stopped_when_the_target_is_gone() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let fixture = Fixture::new();
    // As if the switch-over stopped the source and the copy may still run.
    fixture.container(SOURCE, "exec sleep 300", HostConfig::default(), false);
    let Endpoint::Unix(socket) = endpoint() else {
        panic!("the engine is not on a Unix socket");
    };
    let proxy = Proxy::new(socket);
    let target = Endpoint::Unix(proxy.path.clone());
    let session = DockerSession::connect(&endpoint(), &target).expect("session");
    proxy.cut();

    let error = block_on(session.roll_back_container_as(SOURCE, COPY)).expect_err("refused");
    assert!(
        error.to_string().contains("left the original stopped"),
        "{error}"
    );
    assert_eq!(fixture.running(SOURCE), Some(false));
}

#[test]
#[ignore = "needs a running Docker engine"]
fn refuses_while_a_paused_container_writes_the_volume() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let fixture = Fixture::new();
    let host_config = || HostConfig {
        binds: Some(vec![format!("{VOLUME}:/data")]),
        init: Some(true),
        ..HostConfig::default()
    };
    fixture.container(SOURCE, "exec sleep 300", host_config(), true);
    fixture.container(OTHER, "exec sleep 300", host_config(), true);
    fixture
        .block(fixture.docker.pause_container(OTHER))
        .expect("pause");

    let session = session();
    let volumes = [(VOLUME.to_string(), VOLUME_COPY.to_string())];
    let switched: Result<Vec<TransferEvent>, _> = block_on(
        session
            .switch_container_as(SOURCE, COPY, &volumes)
            .try_collect(),
    );
    let error = switched.expect_err("refused").to_string();
    assert!(error.contains(&format!("{OTHER} also runs")), "{error}");
    assert_eq!(fixture.running(SOURCE), Some(true));
    block_on(session.finish()).expect("finish");
}

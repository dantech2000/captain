//! Runs the Migration Assistant's copies against the real engine that discovery
//! finds, with that one engine as both the source and the target. It touches only
//! what it creates: volumes, a network, a container, and an image tag named
//! `captain-agent-*`, which it removes again even when an assertion fails. It never
//! reads other volumes or images. Ignored by default:
//! `cargo test -p captain-docker --test live_transfer -- --ignored`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use bollard::models::{ContainerCreateBody, HostConfig, NetworkCreateRequest, VolumeCreateRequest};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, ListContainersOptionsBuilder, LogsOptionsBuilder,
    RemoveContainerOptionsBuilder, RemoveImageOptions, RemoveVolumeOptions, TagImageOptionsBuilder,
};
use bollard::{API_DEFAULT_VERSION, Docker};
use captain_core::EngineError;
use captain_core::migration::{MigrationItem, MigrationSession, TransferEvent};
use captain_docker::{DiscoveryInput, DockerSession, Endpoint, discover};
use futures::executor::block_on;
use futures::{StreamExt, TryStreamExt};
use tokio::runtime::Runtime;

const SRC: &str = "captain-agent-src";
const DST: &str = "captain-agent-dst";
const BIG_SRC: &str = "captain-agent-big-src";
const BIG_DST: &str = "captain-agent-big-dst";
const NETWORK: &str = "captain-agent-net";
const CONTAINER: &str = "captain-agent-ctr";
const COPY: &str = "captain-agent-ctr-copy";
const SNAPSHOT: &str = "captain-migrate/captain-agent-ctr:snapshot";
const TAG: &str = "captain-agent-img:1";
const HELPER_IMAGE: &str = "busybox:latest";

/// The tests share one engine and its helper containers, so they run one at a time.
static SERIAL: Mutex<()> = Mutex::new(());

fn endpoint() -> Endpoint {
    discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover")
}

/// Bollard on its own runtime, for setup and checks.
struct Fixture {
    runtime: Runtime,
    docker: Docker,
}

impl Fixture {
    fn new() -> Self {
        let runtime = Runtime::new().expect("runtime");
        let docker = match endpoint() {
            Endpoint::Tcp(address) => Docker::connect_with_http(&address, 120, API_DEFAULT_VERSION),
            other => Docker::connect_with_socket(&other.to_string(), 120, API_DEFAULT_VERSION),
        }
        .expect("bollard");
        let fixture = Self { runtime, docker };
        fixture.clean();
        fixture
    }

    /// Runs `script` in a throwaway `captain-agent-*` busybox container with
    /// `volume` at `/v`, and returns its output.
    fn run(&self, volume: &str, script: &str) -> String {
        let docker = self.docker.clone();
        self.runtime.block_on(async move {
            let body = ContainerCreateBody {
                image: Some(HELPER_IMAGE.into()),
                cmd: Some(vec!["sh".into(), "-c".into(), script.into()]),
                host_config: Some(HostConfig {
                    binds: Some(vec![format!("{volume}:/v")]),
                    ..HostConfig::default()
                }),
                ..ContainerCreateBody::default()
            };
            let options = CreateContainerOptionsBuilder::default()
                .name("captain-agent-run")
                .build();
            let id = docker
                .create_container(Some(options), body)
                .await
                .expect("create")
                .id;
            docker.start_container(&id, None).await.expect("start");
            let _ = docker.wait_container(&id, None).collect::<Vec<_>>().await;
            let logs = LogsOptionsBuilder::default().stdout(true).build();
            let output: Vec<_> = docker
                .logs(&id, Some(logs))
                .try_collect()
                .await
                .expect("logs");
            let force = RemoveContainerOptionsBuilder::default().force(true).build();
            docker.remove_container(&id, Some(force)).await.ok();
            output.into_iter().map(|line| line.to_string()).collect()
        })
    }

    fn create_volume(&self, name: &str) {
        let request = VolumeCreateRequest {
            name: Some(name.into()),
            labels: Some(HashMap::from([("captain-agent-test".into(), "1".into())])),
            ..VolumeCreateRequest::default()
        };
        self.runtime
            .block_on(self.docker.create_volume(request))
            .expect("create volume");
    }

    fn helpers(&self) -> Vec<String> {
        let filters = HashMap::from([("label", vec!["io.captain.migrate"])]);
        let options = ListContainersOptionsBuilder::default()
            .all(true)
            .filters(&filters)
            .build();
        let list = self
            .runtime
            .block_on(self.docker.list_containers(Some(options)));
        list.expect("list")
            .into_iter()
            .flat_map(|c| c.names.unwrap_or_default())
            .collect()
    }

    fn volume_exists(&self, name: &str) -> bool {
        self.runtime
            .block_on(self.docker.inspect_volume(name))
            .is_ok()
    }

    /// Removes everything the tests create.
    fn clean(&self) {
        let docker = self.docker.clone();
        self.runtime.block_on(async move {
            let force = RemoveContainerOptionsBuilder::default().force(true).build();
            for name in ["captain-agent-run", CONTAINER, COPY] {
                docker
                    .remove_container(name, Some(force.clone()))
                    .await
                    .ok();
            }
            for name in [SRC, DST, BIG_SRC, BIG_DST] {
                let options = RemoveVolumeOptions { force: true };
                docker.remove_volume(name, Some(options)).await.ok();
            }
            docker.remove_network(NETWORK).await.ok();
            for image in [TAG, SNAPSHOT] {
                let removed = docker.remove_image(image, None::<RemoveImageOptions>, None);
                removed.await.ok();
            }
        });
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.clean();
    }
}

fn session() -> DockerSession {
    let endpoint = endpoint();
    DockerSession::connect(&endpoint, &endpoint).expect("session")
}

fn collect(
    stream: captain_core::EngineStream<TransferEvent>,
) -> Result<Vec<TransferEvent>, EngineError> {
    block_on(stream.try_collect())
}

#[test]
#[ignore = "needs a running Docker engine"]
fn copies_a_volume_with_owners_and_checks_it() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let fixture = Fixture::new();
    fixture.create_volume(SRC);
    fixture.run(
        SRC,
        "mkdir -p /v/sub && echo hello > /v/sub/a.txt && head -c 70000 /dev/zero > /v/b.bin \
         && ln -s sub/a.txt /v/link && chown 999:999 /v/b.bin && chmod 700 /v/sub",
    );
    let session = session();
    let events = collect(session.copy_volume_as(SRC, DST)).expect("copy");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, TransferEvent::Note(n) if n.starts_with("Checked")))
    );
    let copy = fixture.run(
        DST,
        "cat /v/sub/a.txt; stat -c '%u %s' /v/b.bin; readlink /v/link; stat -c '%a' /v/sub",
    );
    assert_eq!(
        copy.split_whitespace().collect::<Vec<_>>(),
        ["hello", "999", "70000", "sub/a.txt", "700"]
    );
    // A second copy never overwrites the target.
    let again = collect(session.copy_volume_as(SRC, DST)).expect("copy again");
    assert!(matches!(again.last(), Some(TransferEvent::Skipped(_))));
    block_on(session.finish()).expect("finish");
    assert!(fixture.helpers().is_empty());
}

#[test]
#[ignore = "needs a running Docker engine"]
fn stopping_a_copy_removes_the_half_copied_volume() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let fixture = Fixture::new();
    fixture.create_volume(BIG_SRC);
    fixture.run(
        BIG_SRC,
        "for i in $(seq 1 64); do head -c 1048576 /dev/urandom > /v/$i; done",
    );
    let session = session();
    let mut stream = session.copy_volume_as(BIG_SRC, BIG_DST);
    // Wait for the first bytes, then drop the stream, as Stop does.
    let mut started = false;
    while let Some(event) = block_on(stream.next()) {
        if let Ok(TransferEvent::Progress { done, .. }) = event
            && done > 0
        {
            started = true;
            break;
        }
    }
    drop(stream);
    assert!(started, "the copy never started");
    let deadline = Instant::now() + Duration::from_secs(30);
    while (fixture.volume_exists(BIG_DST) || !fixture.helpers().is_empty())
        && Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(!fixture.volume_exists(BIG_DST), "the half copy stays");
    assert!(fixture.helpers().is_empty(), "helpers stay");
}

#[test]
#[ignore = "needs a running Docker engine"]
fn reloads_an_image_with_its_tag() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let fixture = Fixture::new();
    let (repo, tag) = TAG.split_once(':').expect("tag");
    let options = TagImageOptionsBuilder::default()
        .repo(repo)
        .tag(tag)
        .build();
    fixture
        .runtime
        .block_on(fixture.docker.tag_image(HELPER_IMAGE, Some(options)))
        .expect("tag");
    // An ID the target lacks makes the copy export and load for real.
    let item = MigrationItem::Image {
        id: "sha256:captain-agent-missing".into(),
        tags: vec![TAG.into()],
        size: 1,
        in_use: false,
    };
    let session = session();
    let events = collect(session.copy(&item, false)).expect("copy");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, TransferEvent::Progress { done, .. } if *done > 0))
    );
    let loaded = fixture
        .runtime
        .block_on(fixture.docker.inspect_image(TAG))
        .expect("loaded");
    assert!(
        loaded
            .repo_tags
            .unwrap_or_default()
            .contains(&TAG.to_string())
    );
}

#[test]
#[ignore = "needs a running Docker engine"]
fn skips_what_the_target_has_and_scans_the_source() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let fixture = Fixture::new();
    fixture.create_volume(SRC);
    let docker = fixture.docker.clone();
    fixture.runtime.block_on(async move {
        let request = NetworkCreateRequest {
            name: NETWORK.into(),
            ..NetworkCreateRequest::default()
        };
        docker.create_network(request).await.expect("network");
        let body = ContainerCreateBody {
            image: Some(HELPER_IMAGE.into()),
            ..ContainerCreateBody::default()
        };
        let options = CreateContainerOptionsBuilder::default()
            .name(CONTAINER)
            .build();
        docker
            .create_container(Some(options), body)
            .await
            .expect("container");
    });
    let session = session();
    let plan = block_on(session.scan()).expect("scan");
    let keys: Vec<String> = plan.entries.iter().map(|e| e.item.key()).collect();
    assert!(keys.contains(&format!("volume:{SRC}")));
    assert!(keys.contains(&format!("network:{NETWORK}")));
    let container = plan
        .entries
        .iter()
        .find(|e| matches!(&e.item, MigrationItem::Container { name, .. } if name == CONTAINER))
        .expect("container in plan");
    let network = MigrationItem::Network {
        name: NETWORK.into(),
        driver: "bridge".into(),
    };
    for item in [&network, &container.item] {
        let events = collect(session.copy(item, false)).expect("copy");
        assert!(
            matches!(events.last(), Some(TransferEvent::Skipped(_))),
            "{item:?}"
        );
    }
    let free = block_on(session.target_free_space()).expect("free space");
    assert!(free.is_some_and(|bytes| bytes > 0));
    block_on(session.finish()).expect("finish");
    assert!(fixture.helpers().is_empty());
}

#[test]
#[ignore = "needs a running Docker engine"]
fn recreates_a_container_from_a_snapshot() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let fixture = Fixture::new();
    fixture.create_volume(SRC);
    let docker = fixture.docker.clone();
    fixture.runtime.block_on(async move {
        let request = NetworkCreateRequest {
            name: NETWORK.into(),
            ..NetworkCreateRequest::default()
        };
        docker.create_network(request).await.expect("network");
        let body = ContainerCreateBody {
            image: Some(HELPER_IMAGE.into()),
            cmd: Some(vec![
                "sh".into(),
                "-c".into(),
                "echo kept > /marker; sleep 300".into(),
            ]),
            env: Some(vec!["MODE=test".into()]),
            labels: Some(HashMap::from([("captain-agent-test".into(), "1".into())])),
            host_config: Some(HostConfig {
                binds: Some(vec![format!("{SRC}:/data")]),
                network_mode: Some(NETWORK.into()),
                ..HostConfig::default()
            }),
            ..ContainerCreateBody::default()
        };
        let options = CreateContainerOptionsBuilder::default()
            .name(CONTAINER)
            .build();
        docker
            .create_container(Some(options), body)
            .await
            .expect("container");
        docker
            .start_container(CONTAINER, None)
            .await
            .expect("start");
        tokio::time::sleep(Duration::from_millis(500)).await;
    });
    let session = session();
    let events = collect(session.copy_container_as(CONTAINER, COPY, true)).expect("copy");
    // On one engine the copy uses the snapshot, so the snapshot cannot go yet.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, TransferEvent::Note(n) if n.contains(SNAPSHOT)))
    );
    let copy = fixture
        .runtime
        .block_on(fixture.docker.inspect_container(COPY, None))
        .expect("copy");
    let config = copy.config.expect("config");
    assert_eq!(config.image.as_deref(), Some(SNAPSHOT));
    assert!(
        config
            .env
            .unwrap_or_default()
            .contains(&"MODE=test".to_string())
    );
    let labels = config.labels.unwrap_or_default();
    assert_eq!(labels["captain-agent-test"], "1");
    // The mark that lets a switch-over tell Captain's copies from other containers.
    assert!(labels.contains_key("dev.captain.migrated-from"));
    let networks = copy
        .network_settings
        .and_then(|n| n.networks)
        .unwrap_or_default();
    assert!(networks.contains_key(NETWORK));
    assert_eq!(copy.state.and_then(|s| s.running), Some(true));
    let binds = copy.host_config.and_then(|h| h.binds).unwrap_or_default();
    assert_eq!(binds, [format!("{SRC}:/data")]);
}

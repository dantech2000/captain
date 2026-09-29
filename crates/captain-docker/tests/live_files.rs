//! Lists files, previews and saves a file, and runs `top` in a real busybox container.
//! It creates its own container, `captain-agent-files`, and removes it again even
//! when an assertion fails.
//! Ignored by default: `cargo test -p captain-docker --test live_files -- --ignored`.

use std::path::PathBuf;

use bollard::models::ContainerCreateBody;
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, RemoveContainerOptionsBuilder,
};
use bollard::{API_DEFAULT_VERSION, Docker};
use captain_core::ContainerApi;
use captain_core::model::FileKind;
use captain_docker::{DiscoveryInput, DockerEngine, Endpoint, discover};
use futures::StreamExt;
use tokio::runtime::Runtime;

const NAME: &str = "captain-agent-files";
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

/// A fresh, empty folder for saved files.
fn download_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(NAME);
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
#[ignore = "needs a running Docker engine; creates and removes captain-agent-files"]
fn lists_reads_saves_and_tops() {
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    let fixture = Fixture::new(&endpoint);
    // A container left behind by an earlier, killed run.
    fixture.remove();
    let id = fixture.start_sleeper();
    let engine = DockerEngine::connect(endpoint).expect("connect");

    let root = fixture
        .runtime
        .block_on(engine.list_files(&id, "/"))
        .expect("list /");
    let bin = root.iter().find(|e| e.name == "bin").expect("bin");
    assert_eq!(bin.kind, FileKind::Folder);
    // busybox links /lib64 to the folder lib.
    let lib64 = root.iter().find(|e| e.name == "lib64").expect("lib64");
    assert_eq!(lib64.kind, FileKind::Link);
    assert!(lib64.opens);
    assert!(
        fixture
            .runtime
            .block_on(engine.list_files(&id, "/no-such-folder"))
            .is_err()
    );

    let preview = fixture
        .runtime
        .block_on(engine.read_file(&id, "/etc/group", 4))
        .expect("read");
    assert_eq!(preview.bytes, b"root");
    assert!(preview.is_truncated());

    let dir = download_dir();
    let first = fixture
        .runtime
        .block_on(engine.save_path(&id, "/etc/group", &dir))
        .expect("save");
    let second = fixture
        .runtime
        .block_on(engine.save_path(&id, "/etc/group", &dir))
        .expect("save again");
    assert_eq!(first, dir.join("group"));
    assert_eq!(second, dir.join("group (1)"));
    assert!(
        std::fs::read_to_string(first)
            .expect("saved")
            .starts_with("root:")
    );
    let folder = fixture
        .runtime
        .block_on(engine.save_path(&id, "/etc", &dir))
        .expect("save folder");
    assert_eq!(folder, dir.join("etc.tar"));
    std::fs::remove_dir_all(&dir).ok();

    let top = fixture.runtime.block_on(engine.top(&id)).expect("top");
    let cmd = top
        .titles
        .iter()
        .position(|t| t == "CMD")
        .expect("CMD column");
    assert!(
        top.rows.iter().any(|row| row[cmd].contains("sleep 300")),
        "{top:?}"
    );
}

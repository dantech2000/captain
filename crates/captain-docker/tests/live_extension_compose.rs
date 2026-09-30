//! Installs public extensions the way a user types them. Portainer's extension has
//! its Compose file at the image root and serves its page from the backend on
//! `http://localhost:49000`. Docker's Disk Usage extension publishes only version
//! tags, so a reference without a tag must resolve to one. Ignored by default:
//! `cargo test -p captain-docker --test live_extension_compose -- --ignored`.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

use captain_core::ImageApi;
use captain_core::extension::{ExtensionManager, ExtensionPaths, tag_version};
use captain_docker::{DiscoveryInput, DockerEngine, DockerExtensions, discover};
use futures::executor::block_on;

const PORTAINER: &str = "portainer/portainer-docker-extension:2.45.1";
const DISK_USAGE: &str = "docker/disk-usage-extension";

fn connect(name: &str) -> (DockerEngine, DockerExtensions, std::path::PathBuf) {
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    let engine = DockerEngine::connect(endpoint.clone()).expect("connect");
    let dir = std::env::temp_dir().join(name);
    std::fs::remove_dir_all(&dir).ok();
    let manager =
        DockerExtensions::connect(&endpoint, ExtensionPaths::new(dir.clone())).expect("extensions");
    (engine, manager, dir)
}

/// The status line of `GET /` on `127.0.0.1:port`, once the server answers.
fn http_status(port: u16, within: Duration) -> Option<String> {
    let deadline = Instant::now() + within;
    while Instant::now() < deadline {
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            let request =
                format!("GET / HTTP/1.1\r\nHost: localhost:{port}\r\nConnection: close\r\n\r\n");
            let mut answer = String::new();
            if stream.write_all(request.as_bytes()).is_ok()
                && stream.read_to_string(&mut answer).is_ok()
                && answer.starts_with("HTTP/")
            {
                return answer.lines().next().map(str::to_string);
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    None
}

#[test]
#[ignore = "needs a running Docker engine with published ports on the host, the docker CLI, and Docker Hub"]
fn installs_a_compose_backend_from_the_image_root_and_serves_its_page() {
    let (engine, manager, dir) = connect("captain-agent-ext-compose");
    let candidate = block_on(manager.prepare(PORTAINER)).expect("prepare");
    let extension = block_on(manager.install(candidate)).expect("install");
    let compose = manager.paths().compose_dir(&extension.id);
    let status = http_status(49000, Duration::from_secs(90));
    let copied = compose.join("docker-compose.yml").is_file();
    block_on(manager.remove(extension)).expect("remove");
    block_on(engine.remove_image(PORTAINER)).ok();
    std::fs::remove_dir_all(&dir).ok();
    assert!(copied, "the Compose file was not copied");
    let status = status.expect("the backend's page did not answer on localhost:49000");
    assert!(!status.contains(" 5"), "{status}");
}

#[test]
#[ignore = "needs a running Docker engine and Docker Hub"]
fn a_reference_without_a_tag_gets_the_newest_version() {
    let (_engine, manager, dir) = connect("captain-agent-ext-tag");
    let candidate = block_on(manager.prepare(DISK_USAGE)).expect("prepare");
    let tag = candidate
        .image
        .rsplit_once(':')
        .map(|(_, tag)| tag.to_string());
    if candidate.pulled {
        block_on(manager.discard(candidate)).expect("discard");
    }
    std::fs::remove_dir_all(&dir).ok();
    let tag = tag.unwrap_or_default();
    assert!(tag_version(&tag).is_some(), "picked {tag}");
}

#[test]
#[ignore = "needs a running Docker engine and Docker Hub"]
fn a_refused_image_that_prepare_pulled_is_removed_again() {
    const NOT_AN_EXTENSION: &str = "hello-world:latest";
    let (engine, manager, dir) = connect("captain-agent-ext-refused");
    if block_on(engine.inspect_image(NOT_AN_EXTENSION)).is_ok() {
        return;
    }
    assert!(block_on(manager.prepare(NOT_AN_EXTENSION)).is_err());
    let left = block_on(engine.inspect_image(NOT_AN_EXTENSION)).is_ok();
    block_on(engine.remove_image(NOT_AN_EXTENSION)).ok();
    std::fs::remove_dir_all(&dir).ok();
    assert!(!left, "the pulled image stayed");
}

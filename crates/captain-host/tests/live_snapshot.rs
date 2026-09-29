//! Snapshots a real Captain Engine VM: save, change, restore, and delete. Like
//! `live.rs`, it needs Lima 2.2 and downloads about 600 MB, so it is ignored by
//! default. Run it with:
//!
//! ```sh
//! cargo test -p captain-host --test live_snapshot -- --ignored --nocapture
//! ```
//!
//! It uses a VM named `captain-agent-test` in a temporary `LIMA_HOME`. The
//! snapshots go to the `snapshots` folder next to it, never to `~/.captain`.
#![cfg(target_os = "macos")]

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use captain_core::{EngineHost, GIB, HostResources};
use captain_host::{LimaHost, LimaPaths};
use futures::StreamExt;
use futures::executor::block_on;

const VOLUME: &str = "captain-agent-snap";

/// Deletes the VM and the temporary folder when the test ends.
struct Cleanup {
    host: LimaHost,
    root: PathBuf,
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        if let Err(error) = block_on(self.host.reset()) {
            eprintln!("reset failed: {error}");
        }
        std::fs::remove_dir_all(&self.root).ok();
    }
}

#[test]
#[ignore = "creates a real Lima VM; the first run downloads about 600 MB"]
fn restores_a_snapshot_of_a_real_engine() {
    let root = PathBuf::from(format!("/tmp/captain-snap-{}", std::process::id()));
    let paths = LimaPaths {
        lima_home: root.join("lima"),
        instance: "captain-agent-test".into(),
    };
    let resources = HostResources {
        cpus: 2,
        memory_bytes: 4 * GIB,
        disk_bytes: 16 * GIB,
    };
    let host = LimaHost::with_paths(paths.clone(), resources);
    let _cleanup = Cleanup {
        host: host.clone(),
        root: root.clone(),
    };
    let snapshots = host.snapshots().expect("Lima has snapshots");
    let socket = paths.docker_socket();

    start(&host);
    block_on(host.stop()).unwrap();
    let base = block_on(snapshots.create("base".into(), "clean".into())).unwrap();
    assert!(root.join("snapshots").join(&base.id).join("disk").is_file());

    start(&host);
    let body = format!(r#"{{"Name":"{VOLUME}"}}"#);
    request(&socket, "POST /volumes/create", &body);
    assert!(volumes(&socket).contains(VOLUME));
    assert!(
        block_on(snapshots.restore(base.id.clone())).is_err(),
        "engine runs"
    );

    block_on(host.stop()).unwrap();
    block_on(snapshots.restore(base.id.clone())).unwrap();
    start(&host);
    assert!(!volumes(&socket).contains(VOLUME));

    block_on(snapshots.delete(base.id)).unwrap();
    assert!(block_on(snapshots.list()).unwrap().snapshots.is_empty());
    block_on(host.stop()).unwrap();
}

fn start(host: &LimaHost) {
    let mut progress = host.start();
    while let Some(line) = block_on(progress.next()) {
        if let Err(error) = line {
            panic!("start failed: {error}");
        }
    }
}

fn volumes(socket: &Path) -> String {
    request(socket, "GET /volumes", "")
}

/// One HTTP/1.0 request with a JSON `body` on the Docker socket. Fails unless the
/// status is 2xx.
fn request(socket: &Path, line: &str, body: &str) -> String {
    let mut stream = UnixStream::connect(socket).expect("connect to the Docker socket");
    let head = format!(
        "{line} HTTP/1.0\r\nHost: docker\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(head.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let status = response.lines().next().unwrap_or_default();
    assert!(status.contains(" 20"), "{response}");
    response
}

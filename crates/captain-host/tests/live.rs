//! Starts a real Captain Engine VM. It needs Lima 2.2 or newer, and the first run
//! downloads an Ubuntu image (about 600 MB) and installs Docker, so it is ignored by
//! default. Run it with:
//!
//! ```sh
//! cargo test -p captain-host --test live -- --ignored --nocapture
//! ```
//!
//! It uses a VM named `captain-agent-test` in a temporary `LIMA_HOME`, and deletes
//! both at the end, also when it fails. It never touches `~/.lima` or `~/.captain`.
#![cfg(target_os = "macos")]

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use captain_core::{EngineHost, GIB, HostResources, HostStatus};
use captain_host::{LimaHost, LimaPaths};
use futures::StreamExt;
use futures::executor::block_on;

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
#[ignore = "creates and starts a real Lima VM; the first run downloads about 600 MB"]
fn starts_checks_and_stops_a_real_engine() {
    // A short folder: the macOS temp folder is too long for Lima's sockets.
    let root = PathBuf::from(format!("/tmp/captain-live-{}", std::process::id()));
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

    assert_eq!(block_on(host.status()).unwrap(), HostStatus::NotCreated);

    let mut progress = host.start();
    while let Some(line) = block_on(progress.next()) {
        match line {
            Ok(line) => eprintln!("progress: {line}"),
            Err(error) => panic!("start failed: {error}"),
        }
    }
    assert_eq!(block_on(host.status()).unwrap(), HostStatus::Running);

    let endpoint = host.endpoint().unwrap();
    let socket = endpoint.strip_prefix("unix://").unwrap();
    assert_eq!(Path::new(socket), paths.docker_socket());
    let version = docker_version(Path::new(socket));
    eprintln!("docker version: {version}");
    assert!(version.contains("\"ApiVersion\""), "{version}");

    block_on(host.stop()).unwrap();
    assert_eq!(block_on(host.status()).unwrap(), HostStatus::Stopped);
}

/// `GET /version` on the Docker socket: what `docker version` asks the engine.
fn docker_version(socket: &Path) -> String {
    let mut stream = UnixStream::connect(socket).expect("connect to the Docker socket");
    stream
        .write_all(b"GET /version HTTP/1.0\r\nHost: docker\r\n\r\n")
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let status = response.lines().next().unwrap_or_default();
    assert!(status.contains(" 200 "), "{response}");
    response
}

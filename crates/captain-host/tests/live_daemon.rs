//! Applies Docker daemon settings to a real Captain Engine VM: a registry mirror, a
//! custom key, and the TCP socket, then an invalid file, then the defaults. Like
//! `live.rs`, it needs Lima 2.2 and downloads about 600 MB, so it is ignored by
//! default. It also needs the `docker` CLI on `PATH` or in `~/.rd/bin`. Run it with:
//!
//! ```sh
//! cargo test -p captain-host --test live_daemon -- --ignored --nocapture
//! ```
//!
//! It uses a VM named `captain-agent-daemon` in `~/.clo/lima` (a short path, for
//! Lima's sockets), and deletes both at the end. Set `CAPTAIN_KEEP_VM=1` to keep the
//! VM running for another check.
#![cfg(target_os = "macos")]

use std::path::PathBuf;
use std::process::Command;

use captain_core::daemon::DaemonSettings;
use captain_core::{EngineHost, GIB, HostResources};
use captain_host::{LimaHost, LimaPaths};
use futures::StreamExt;
use futures::executor::block_on;
use serde_json::json;

const MIRROR: &str = "https://mirror.gcr.io";
const PORT: u16 = 23750;

/// Deletes the VM and its folder when the test ends, unless `CAPTAIN_KEEP_VM` is set.
struct Cleanup {
    host: LimaHost,
    root: PathBuf,
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        if std::env::var_os("CAPTAIN_KEEP_VM").is_some() {
            return;
        }
        if let Err(error) = block_on(self.host.reset()) {
            eprintln!("reset failed: {error}");
        }
        std::fs::remove_dir_all(&self.root).ok();
    }
}

#[test]
#[ignore = "creates a real Lima VM; the first run downloads about 600 MB"]
fn applies_checks_and_reverts_daemon_settings() {
    let home = PathBuf::from(std::env::var("HOME").expect("HOME"));
    let root = home.join(".clo");
    let paths = LimaPaths {
        lima_home: root.join("lima"),
        instance: "captain-agent-daemon".into(),
    };
    let resources = HostResources {
        cpus: 2,
        memory_bytes: 4 * GIB,
        disk_bytes: 16 * GIB,
    };
    let host = LimaHost::with_paths(paths.clone(), resources);
    let _cleanup = Cleanup {
        host: host.clone(),
        root,
    };
    let socket = format!("unix://{}", paths.docker_socket().display());

    let mut custom = serde_json::Map::new();
    custom.insert("log-level".into(), json!("warn"));
    let settings = DaemonSettings {
        registry_mirrors: vec![MIRROR.into()],
        custom,
        tcp: true,
        tcp_port: PORT,
        ..DaemonSettings::default()
    };
    host.set_daemon(settings.clone());
    restart(&host).expect("start with the settings");
    let again = restart(&host).expect("start again");
    assert!(
        !again.iter().any(|l| l.starts_with("Applying")),
        "{again:?}"
    );
    assert_eq!(host.running_daemon(), Some(settings.state()));
    let info = docker(
        &socket,
        &["info", "--format", "{{json .RegistryConfig.Mirrors}}"],
    );
    assert!(info.contains(MIRROR), "{info}");
    let tcp = format!("tcp://127.0.0.1:{PORT}");
    docker(&tcp, &["version", "--format", "{{.Server.Version}}"]);

    let mut bogus = settings.clone();
    bogus.custom.insert("captain-bogus".into(), json!(1));
    host.set_daemon(bogus);
    let error = restart(&host).expect_err("dockerd rejects the key");
    assert!(error.contains("captain-bogus"), "{error}");
    assert_eq!(host.running_daemon(), Some(settings.state()));
    docker(&socket, &["version", "--format", "{{.Server.Version}}"]);

    host.set_daemon(DaemonSettings::default());
    restart(&host).expect("start with the defaults");
    assert_eq!(
        host.running_daemon(),
        Some(DaemonSettings::default().state())
    );
    let info = docker(
        &socket,
        &["info", "--format", "{{json .RegistryConfig.Mirrors}}"],
    );
    assert!(!info.contains(MIRROR), "{info}");
    assert!(
        try_docker(&tcp, &["version"]).is_err(),
        "the TCP socket is closed"
    );
}

/// Stops the engine if it runs, then starts it. Returns the progress lines, or the
/// first error.
fn restart(host: &LimaHost) -> Result<Vec<String>, String> {
    block_on(host.stop()).map_err(|error| error.0)?;
    let mut lines = Vec::new();
    let mut progress = host.start();
    while let Some(line) = block_on(progress.next()) {
        let line = line.map_err(|error| error.0)?;
        eprintln!("{line}");
        lines.push(line);
    }
    Ok(lines)
}

fn docker(host: &str, args: &[&str]) -> String {
    try_docker(host, args).unwrap_or_else(|error| panic!("docker {args:?}: {error}"))
}

fn try_docker(host: &str, args: &[&str]) -> Result<String, String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let path = format!(
        "{home}/.rd/bin:{}",
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new("docker")
        .args(args)
        .env("PATH", path)
        .env("DOCKER_HOST", host)
        .env_remove("DOCKER_CONTEXT")
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

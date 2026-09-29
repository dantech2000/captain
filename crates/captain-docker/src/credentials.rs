//! Finds the registry login for a push the way the Docker CLI does: the config
//! file, then a credential helper. See docs/features/0019-image-build-push-scan.md.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use captain_core::registry::{CredentialSource, DockerConfig, RegistryAuth};

use crate::compose::locate_helper;

/// How long a credential helper may take. A keychain can ask the user first.
const HELPER_TIMEOUT: Duration = Duration::from_secs(30);

/// The login for `host`, for example `docker.io` or `ghcr.io`, or `None` when the
/// Docker config has none. Blocks while a helper runs.
pub fn registry_login(host: &str) -> Option<RegistryAuth> {
    let path = config_path()?;
    let json = std::fs::read_to_string(&path).ok()?;
    let config = match DockerConfig::parse(&json) {
        Ok(config) => config,
        Err(error) => {
            tracing::warn!(%error, path = %path.display(), "cannot read the Docker config");
            return None;
        }
    };
    match config.source(host) {
        CredentialSource::Helper {
            name,
            server_address,
        } => run_helper(&name, &server_address),
        CredentialSource::Stored(login) => Some(login),
        CredentialSource::None => None,
    }
}

/// `$DOCKER_CONFIG/config.json`, else `~/.docker/config.json`.
fn config_path() -> Option<PathBuf> {
    let dir = match std::env::var_os("DOCKER_CONFIG") {
        Some(dir) => PathBuf::from(dir),
        None => std::env::home_dir()?.join(".docker"),
    };
    Some(dir.join("config.json"))
}

/// Runs `docker-credential-<name> get` with `server_address` on stdin.
fn run_helper(name: &str, server_address: &str) -> Option<RegistryAuth> {
    let binary = format!("docker-credential-{name}");
    let path = std::env::var_os("PATH");
    let home = std::env::home_dir();
    let Some(helper) = locate_helper(&binary, path.as_deref(), home.as_deref(), Path::is_file)
    else {
        tracing::warn!(%binary, "the credential helper is not installed");
        return None;
    };
    let server = server_address.to_string();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let output = Command::new(helper)
            .arg("get")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(server.as_bytes())?;
                }
                child.wait_with_output()
            });
        tx.send(output).ok();
    });
    let output = match rx.recv_timeout(HELPER_TIMEOUT) {
        Ok(Ok(output)) => output,
        Ok(Err(error)) => {
            tracing::warn!(%error, %binary, "the credential helper failed");
            return None;
        }
        Err(_) => {
            tracing::warn!(%binary, "the credential helper did not answer");
            return None;
        }
    };
    // A helper without a login exits with an error such as "credentials not found".
    if !output.status.success() {
        return None;
    }
    RegistryAuth::from_helper(&String::from_utf8_lossy(&output.stdout), server_address)
}

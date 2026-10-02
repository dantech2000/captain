//! Where Captain Engine keeps its files.
//!
//! `LIMA_HOME` is `~/.captain/lima`, not `~/Library/Application Support/Captain/lima`.
//! Lima puts Unix sockets in the instance folder, and on macOS a socket path must be
//! shorter than 104 bytes. Lima avoids `Application Support` for the same reason
//! (https://lima-vm.io/docs/dev/internals/), and Rancher Desktop fails for long user
//! names with it (rancher-sandbox/rancher-desktop#797).

use std::path::{Path, PathBuf};

/// The Lima instance name.
pub const INSTANCE: &str = "captain";

/// `sockaddr_un` holds this many bytes on macOS, including the final NUL.
#[cfg(target_os = "linux")]
const UNIX_PATH_MAX: usize = 108;
#[cfg(not(target_os = "linux"))]
const UNIX_PATH_MAX: usize = 104;

/// The longest socket name that Lima creates in an instance folder
/// (`filenames.LongestSock` in Lima: an SSH control socket with a random suffix).
const LONGEST_SOCKET: &str = "ssh.sock.1234567890123456";

/// The files of one Captain Engine instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimaPaths {
    /// The `LIMA_HOME` that every `limactl` call gets.
    pub lima_home: PathBuf,
    /// The Lima instance name.
    pub instance: String,
}

impl LimaPaths {
    /// `~/.captain/lima` under `home`.
    pub fn for_home(home: &Path) -> Self {
        Self {
            lima_home: home.join(".captain").join("lima"),
            instance: INSTANCE.into(),
        }
    }

    pub fn instance_dir(&self) -> PathBuf {
        self.lima_home.join(&self.instance)
    }

    /// The host end of the forwarded Docker socket.
    pub fn docker_socket(&self) -> PathBuf {
        self.instance_dir().join("sock").join("docker.sock")
    }

    /// The template Captain writes before `limactl create`. It sits next to
    /// `LIMA_HOME`, not in it, so Lima never mistakes it for an instance.
    pub fn template_file(&self) -> PathBuf {
        self.lima_home
            .parent()
            .unwrap_or(&self.lima_home)
            .join(format!("{}-engine.yaml", self.instance))
    }

    /// The lock that a process holds while it starts, stops, or changes the
    /// instance, next to the template. See docs/features/0022-command-line.md.
    pub fn lock_file(&self) -> PathBuf {
        self.template_file().with_extension("lock")
    }

    /// Lima's shared folder with the SSH key pair.
    pub fn config_dir(&self) -> PathBuf {
        self.lima_home.join("_config")
    }

    /// Where snapshots live, one folder each, next to `LIMA_HOME` on the same volume
    /// so the disk can be cloned. See docs/adr/0012-snapshots.md.
    pub fn snapshots_dir(&self) -> PathBuf {
        self.lima_home
            .parent()
            .unwrap_or(&self.lima_home)
            .join("snapshots")
    }

    /// Downloads, next to `LIMA_HOME`: `~/.captain/cache`. The VM sees it through the
    /// home mount. See ADR 0010.
    pub fn cache_dir(&self) -> PathBuf {
        self.lima_home
            .parent()
            .unwrap_or(&self.lima_home)
            .join("cache")
    }

    /// One folder per downloaded k3s version.
    pub fn k3s_cache(&self) -> PathBuf {
        self.cache_dir().join("k3s")
    }

    /// The cached k3s version list.
    pub fn k3s_versions_file(&self) -> PathBuf {
        self.cache_dir().join("k3s-versions.json")
    }

    /// Captain's own kubeconfig, with only the `captain-desktop` context.
    pub fn kubeconfig(&self) -> PathBuf {
        self.lima_home
            .parent()
            .unwrap_or(&self.lima_home)
            .join("kubeconfig")
    }

    /// k3s's certificate authority, which the host port check trusts.
    pub fn kubernetes_ca(&self) -> PathBuf {
        self.kubeconfig().with_file_name("kubernetes-ca.pem")
    }

    /// Fails with a message if a socket in the instance folder would be too long.
    pub fn check_socket_paths(&self) -> Result<(), String> {
        let longest = self.instance_dir().join(LONGEST_SOCKET);
        let len = longest.as_os_str().len();
        if len >= UNIX_PATH_MAX {
            return Err(format!(
                "The path {} is {len} bytes long. Unix sockets need fewer than {UNIX_PATH_MAX}.",
                longest.display()
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

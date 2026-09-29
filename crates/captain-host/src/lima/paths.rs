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

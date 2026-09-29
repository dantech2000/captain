//! The Lima template for Captain Engine. It follows Lima's `docker-rootful` template
//! (templates/docker-rootful.yaml in lima-vm/lima): an Ubuntu LTS guest with rootful
//! `dockerd` and the containerd snapshotter, and the Docker socket forwarded to
//! `<instance dir>/sock/docker.sock`. See ADR 0008.

use captain_core::HostResources;
use captain_core::diagnostics::MINIMUM_LIMA_VERSION;

/// The rootful Docker setup, copied from `docker-rootful.yaml`.
const DOCKER: &str = r#"# containerd is managed by Docker, not by Lima.
containerd:
  system: false
  user: false
provision:
- mode: system
  script: |
    #!/bin/sh
    sed -i 's/host.lima.internal.*/host.lima.internal host.docker.internal/' /etc/hosts
- mode: system
  script: |
    #!/bin/bash
    set -eux -o pipefail
    command -v docker >/dev/null 2>&1 && exit 0
    export DEBIAN_FRONTEND=noninteractive
    curl -fsSL https://get.docker.com | sh
- mode: yq
  path: /etc/systemd/system/docker.socket.d/override.conf
  format: ini
  expression: .Socket.SocketUser="{{.User}}"
- mode: yq
  path: "/etc/docker/daemon.json"
  expression: |
    .features.cdi = true |
    .features.containerd-snapshotter = {{.Param.containerdSnapshotter}}
probes:
- script: |
    #!/bin/bash
    set -eux -o pipefail
    if ! timeout 30s bash -c "until command -v docker >/dev/null 2>&1; do sleep 3; done"; then
      echo >&2 "docker is not installed yet"
      exit 1
    fi
    if ! timeout 30s bash -c "until pgrep dockerd; do sleep 3; done"; then
      echo >&2 "dockerd is not running"
      exit 1
    fi
  hint: See "/var/log/cloud-init-output.log" in the guest
hostResolver:
  hosts:
    host.docker.internal: host.lima.internal
portForwards:
- guestSocket: "/var/run/docker.sock"
  hostSocket: "{{.Dir}}/sock/docker.sock"
param:
  containerdSnapshotter: true
"#;

/// The template for `resources`. `rosetta` turns on Rosetta for x86_64 containers,
/// which only works on Apple Silicon.
pub fn render(resources: &HostResources, rosetta: bool) -> String {
    let mut yaml = format!(
        "# Captain Engine. Captain writes this file before it creates the VM.\n\
         minimumLimaVersion: {MINIMUM_LIMA_VERSION}\n\
         base:\n\
         - template:_images/ubuntu-lts\n\
         vmType: vz\n\
         mountType: virtiofs\n\
         cpus: {cpus}\n\
         memory: {memory}\n\
         disk: {disk}\n",
        cpus = resources.cpus,
        memory = memory_size(resources.memory_bytes),
        disk = disk_size(resources.disk_bytes),
    );
    if rosetta {
        yaml.push_str("vmOpts:\n  vz:\n    rosetta:\n      enabled: true\n      binfmt: true\n");
    }
    // The home folder is writable so bind mounts from projects work.
    yaml.push_str(
        "mounts:\n\
         - location: \"~\"\n  writable: true\n\
         - location: \"{{.GlobalTempDir}}/lima\"\n  mountPoint: /tmp/lima\n  writable: true\n",
    );
    yaml.push_str(DOCKER);
    yaml
}

/// Memory in whole MiB, for example `4608MiB`, so a quarter of an odd total works.
pub fn memory_size(bytes: u64) -> String {
    format!("{}MiB", (bytes / (1024 * 1024)).max(1))
}

/// Disk in whole GiB, for example `64GiB`.
pub fn disk_size(bytes: u64) -> String {
    format!("{}GiB", (bytes / captain_core::GIB).max(1))
}

#[cfg(test)]
mod tests;

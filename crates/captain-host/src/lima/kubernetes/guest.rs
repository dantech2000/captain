//! The shell scripts that run k3s in the VM over `limactl shell`, and what they
//! print. k3s runs as a systemd unit with Docker as its runtime. See ADR 0010.

use captain_core::kubernetes::{KubernetesSettings, KubernetesStatus};

use crate::lima::daemon::shell;

/// Where Captain notes the installed k3s version, to refuse a downgrade.
const VERSION_FILE: &str = "/var/lib/rancher/k3s/captain-version";

/// Installs k3s from the host's cache folder `$1` (binary `$2`, images `$3`) as
/// version `$4`, with port `$5` and Traefik `$6` (`1` or `0`). It restarts k3s when
/// the binary or the unit changed, and starts it otherwise. It does not wait.
/// cri-dockerd forwards ports with `socat` (streaming_others.go in
/// Mirantis/cri-dockerd), which the Ubuntu image lacks, so it installs it once.
const INSTALL_SCRIPT: &str = r#"set -eu
dir=$1 bin=$2 images=$3 version=$4 port=$5 traefik=$6
mkdir -p /var/lib/rancher/k3s
if ! command -v socat >/dev/null; then
  export DEBIAN_FRONTEND=noninteractive
  apt-get -qq update >/dev/null && apt-get -qq install -y socat >/dev/null
fi
changed=0
if ! cmp -s "$dir/$bin" /usr/local/bin/k3s; then
  systemctl stop k3s 2>/dev/null || true
  install -m 755 "$dir/$bin" /usr/local/bin/k3s
  changed=1
fi
mark=/var/lib/rancher/k3s/captain-images
if [ "$(cat "$mark" 2>/dev/null)" != "$version" ]; then
  docker load -i "$dir/$images" >/dev/null
  echo "$version" > "$mark"
fi
flags="--docker --https-listen-port $port --write-kubeconfig-mode 644"
[ "$traefik" = 1 ] || flags="$flags --disable traefik"
unit=/etc/systemd/system/k3s.service
cat > "$unit.captain-new" <<UNIT
[Unit]
Description=Kubernetes (k3s) for Captain
After=docker.service network-online.target
Wants=docker.service network-online.target

[Service]
Type=notify
ExecStart=/usr/local/bin/k3s server $flags
KillMode=process
Delegate=yes
LimitNOFILE=1048576
LimitNPROC=infinity
LimitCORE=infinity
TasksMax=infinity
TimeoutStartSec=0
Restart=always
RestartSec=5s

[Install]
WantedBy=multi-user.target
UNIT
if cmp -s "$unit.captain-new" "$unit"; then rm -f "$unit.captain-new"; else mv "$unit.captain-new" "$unit"; changed=1; fi
echo "$version" > /var/lib/rancher/k3s/captain-version
systemctl daemon-reload
systemctl enable k3s >/dev/null 2>&1
if [ "$changed" = 1 ]; then systemctl restart --no-block k3s; else systemctl start --no-block k3s; fi
"#;

/// Prints `ready` once systemd has the unit up (k3s notifies it after its agent
/// starts), the API server answers, and the `default` service account exists, so a
/// first `kubectl apply` works.
const READY_SCRIPT: &str = r#"systemctl is-active -q k3s \
  && test -f /etc/rancher/k3s/k3s.yaml \
  && k3s kubectl get --raw /readyz >/dev/null 2>&1 \
  && k3s kubectl -n default get serviceaccount default >/dev/null 2>&1 \
  && echo ready || true"#;

/// Deletes Traefik's charts, which `--disable traefik` leaves in place
/// (k3s-io/k3s#5103).
const REMOVE_TRAEFIK_SCRIPT: &str = r#"k3s kubectl -n kube-system delete helmchart traefik traefik-crd --ignore-not-found >/dev/null"#;

/// Stops k3s and the pod containers, and keeps it from starting at boot. The pod
/// containers are Docker containers, so they outlive k3s otherwise.
const DISABLE_SCRIPT: &str = r#"set -u
[ -f /etc/systemd/system/k3s.service ] || exit 0
systemctl disable --now k3s >/dev/null 2>&1 || true
ids=$(docker ps -q --filter label=io.kubernetes.pod.namespace)
[ -z "$ids" ] || docker stop $ids >/dev/null
"#;

/// Deletes the cluster state, the list Rancher Desktop deletes in `deleteKubeState`,
/// and the pod containers. Images stay.
const RESET_SCRIPT: &str = r#"set -u
systemctl disable --now k3s >/dev/null 2>&1 || true
ids=$(docker ps -aq --filter label=io.kubernetes.pod.namespace)
[ -z "$ids" ] || docker rm -f $ids >/dev/null
for m in $(awk '$2 ~ "^/(var/lib/kubelet|run/k3s)" {print $2}' /proc/self/mounts | sort -r); do umount "$m" 2>/dev/null || umount -l "$m" || true; done
rm -rf /var/lib/kubelet /var/lib/rancher/k3s/data /var/lib/rancher/k3s/server /var/lib/rancher/k3s/storage /etc/rancher/k3s /run/k3s /var/lib/rancher/k3s/captain-version
"#;

/// Prints the unit state, then the installed version (or nothing).
const STATUS_SCRIPT: &str = r#"systemctl is-active k3s 2>/dev/null || true
cat /var/lib/rancher/k3s/captain-version 2>/dev/null || true
"#;

fn sudo(instance: &str, script: &str, args: &[String]) -> Vec<String> {
    let mut all = shell(instance, script, &["sudo", "-n"]);
    all.extend(args.iter().cloned());
    all
}

/// Arguments for [`INSTALL_SCRIPT`]: the version's cache folder and file names.
pub fn install_args(
    instance: &str,
    folder: &str,
    binary: &str,
    images: &str,
    settings: &KubernetesSettings,
    version: &str,
) -> Vec<String> {
    let traefik = if settings.traefik { "1" } else { "0" };
    let args = [
        folder,
        binary,
        images,
        version,
        &settings.port.to_string(),
        traefik,
    ];
    sudo(instance, INSTALL_SCRIPT, &args.map(String::from))
}

pub fn ready_args(instance: &str) -> Vec<String> {
    sudo(instance, READY_SCRIPT, &[])
}

pub fn remove_traefik_args(instance: &str) -> Vec<String> {
    sudo(instance, REMOVE_TRAEFIK_SCRIPT, &[])
}

pub fn disable_args(instance: &str) -> Vec<String> {
    sudo(instance, DISABLE_SCRIPT, &[])
}

pub fn reset_args(instance: &str) -> Vec<String> {
    sudo(instance, RESET_SCRIPT, &[])
}

pub fn status_args(instance: &str) -> Vec<String> {
    sudo(instance, STATUS_SCRIPT, &[])
}

pub fn kubeconfig_args(instance: &str) -> Vec<String> {
    sudo(instance, "cat /etc/rancher/k3s/k3s.yaml", &[])
}

/// `cat` of [`VERSION_FILE`], for the downgrade check.
pub fn version_args(instance: &str) -> Vec<String> {
    sudo(
        instance,
        &format!("cat {VERSION_FILE} 2>/dev/null || true"),
        &[],
    )
}

/// Reads the output of [`status_args`].
pub fn parse_status(output: &str) -> KubernetesStatus {
    let mut lines = output.lines().map(str::trim);
    let state = lines.next().unwrap_or_default();
    let version = lines.next().unwrap_or_default().to_string();
    match state {
        "active" => KubernetesStatus::Running { version },
        "activating" | "reloading" => KubernetesStatus::Starting,
        "failed" => KubernetesStatus::Failed(
            "k3s stopped with an error. See `journalctl -u k3s` in the VM.".into(),
        ),
        _ => KubernetesStatus::Off,
    }
}

#[cfg(test)]
mod tests;

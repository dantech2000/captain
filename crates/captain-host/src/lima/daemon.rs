//! Reads and writes the Docker daemon settings in the VM over `limactl shell`: the
//! file `/etc/docker/daemon.json`, and a systemd drop-in that adds the TCP listener.
//! See feature 0020.
//!
//! The TCP listener is a `-H` flag in a drop-in, next to the unit's own `-H fd://`,
//! as Docker's remote access docs show. `daemon.json` never has `hosts`, because
//! `hosts` there and `-H` in the unit stop `dockerd` (moby/moby#22339). `dockerd`
//! listens on the guest's 127.0.0.1, and Lima's default port forwarding rule forwards
//! that port to the same port on the host's 127.0.0.1.

use captain_core::daemon::DaemonState;
use serde_json::Value;

/// Separates `daemon.json` from the drop-in in the output of [`READ_SCRIPT`].
const MARKER: &str = "#captain-tcp";

/// Prints `daemon.json` (or `{}`), the marker, and the drop-in, if any.
const READ_SCRIPT: &str = r#"cat /etc/docker/daemon.json 2>/dev/null || echo '{}'
echo
echo '#captain-tcp'
cat /etc/systemd/system/docker.service.d/captain-tcp.conf 2>/dev/null || true
"#;

/// Installs `daemon.json` from standard input and the drop-in for port `$1` (`0`
/// for none), then restarts Docker. `dockerd --validate` checks the file first, and a
/// failed restart puts the old files back.
const WRITE_SCRIPT: &str = r#"set -eu
json=/etc/docker/daemon.json
unit=/etc/systemd/system/docker.service.d/captain-tcp.conf
mkdir -p /etc/docker /etc/systemd/system/docker.service.d
cat > "$json.captain-new"
if ! out=$(dockerd --validate --config-file="$json.captain-new" 2>&1); then
  rm -f "$json.captain-new"
  echo "$out" >&2
  exit 1
fi
backup() { if [ -f "$1" ]; then cp -p "$1" "$1.captain-old"; else rm -f "$1.captain-old"; fi; }
restore() { if [ -f "$1.captain-old" ]; then mv "$1.captain-old" "$1"; else rm -f "$1"; fi; }
backup "$json"
backup "$unit"
mv "$json.captain-new" "$json"
if [ "$1" -gt 0 ]; then
  start=$(sed -n 's/^ExecStart=//p' /lib/systemd/system/docker.service | head -n 1)
  [ -n "$start" ] || { restore "$json"; restore "$unit"; echo "docker.service has no ExecStart line." >&2; exit 1; }
  printf '[Service]\nExecStart=\nExecStart=%s -H tcp://127.0.0.1:%s\n' "$start" "$1" > "$unit"
else
  rm -f "$unit"
fi
systemctl daemon-reload
if ! systemctl restart docker; then
  why=$(journalctl -u docker -n 1 --no-pager -o cat 2>/dev/null || true)
  restore "$json"
  restore "$unit"
  systemctl daemon-reload
  systemctl reset-failed docker || true
  systemctl restart docker || true
  echo "Docker did not start with the new settings. $why" >&2
  exit 1
fi
rm -f "$json.captain-old" "$unit.captain-old"
"#;

/// `limactl shell` arguments that print the daemon files of `instance`.
pub fn read_args(instance: &str) -> Vec<String> {
    shell(instance, READ_SCRIPT, &[])
}

/// `limactl shell` arguments that install `state` in `instance`. Pass
/// [`write_input`] on standard input.
pub fn write_args(instance: &str, state: &DaemonState) -> Vec<String> {
    let port = state.tcp_port.unwrap_or(0).to_string();
    shell(instance, WRITE_SCRIPT, &["sudo", "-n"])
        .into_iter()
        .chain([port])
        .collect()
}

/// The `daemon.json` text for `state`.
pub fn write_input(state: &DaemonState) -> String {
    let mut json = serde_json::to_string_pretty(&state.daemon_json).unwrap_or_default();
    json.push('\n');
    json
}

/// `limactl shell --workdir / <instance> [prefix] sh -c <script> sh`; arguments
/// after the last `sh` become `$1` and on.
pub(super) fn shell(instance: &str, script: &str, prefix: &[&str]) -> Vec<String> {
    let mut args: Vec<String> = ["shell", "--workdir", "/", instance]
        .map(String::from)
        .to_vec();
    args.extend(prefix.iter().map(ToString::to_string));
    args.extend(["sh", "-c", script, "sh"].map(String::from));
    args
}

/// Reads the output of [`read_args`]. `None` if `daemon.json` is not valid JSON.
pub fn parse_state(output: &str) -> Option<DaemonState> {
    let (json, unit) = output.split_once(MARKER).unwrap_or((output, ""));
    let daemon_json: Value = serde_json::from_str(json.trim()).ok()?;
    let tcp_port = unit.split("-H tcp://127.0.0.1:").nth(1).and_then(|rest| {
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        digits.parse().ok()
    });
    Some(DaemonState {
        daemon_json,
        tcp_port,
    })
}

#[cfg(test)]
mod tests;

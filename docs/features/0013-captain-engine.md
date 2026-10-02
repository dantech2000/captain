# Feature 0013: Captain Engine

- Milestone: M12
- Status: Done on macOS (M12). Later changes: Start, Stop, and Restart moved from the sidebar to the Diagnostics page ([0016](0016-diagnostics.md)), and the Settings card became the Engine section of [0037](0037-settings-page.md).
- Design: [ADR 0008](../adr/0008-captain-engine.md)

## Goal

Captain starts, stops, and configures its own Docker engine, so it works without Rancher Desktop, Docker Desktop, or OrbStack. On macOS the engine runs in a Lima VM that only Captain manages.

## In scope

- `EngineHost` in `captain-core`: status, start with progress lines, stop, the endpoint, resources, and reset. Default resources come from the computer's CPUs and memory.
- A new crate, `captain-host`:
  - `LimaHost` (macOS): one Lima instance named `captain`, created from a template that Captain writes. The template follows Lima's `docker-rootful` template: an Ubuntu LTS guest, `vz`, virtiofs, Rosetta on Apple Silicon, `~` and `/tmp/lima` mounted writable, rootful `dockerd` with the containerd snapshotter, and the Docker socket forwarded to `<instance>/sock/docker.sock`.
  - `SystemHost` (Linux): reports the system `dockerd` socket and does not start or stop it.
  - `UnavailableHost` (Windows, for now): always "not installed".
- Settings: an engine choice, **Captain Engine** or **Other engine**, and "Stop the engine when Captain quits" (on by default). The choice defaults to Captain Engine when `limactl` exists and no custom endpoint was saved. Old settings files still load.
- Launch: with Captain Engine, a running engine is connected; a stopped engine starts, with progress; if no engine exists yet, the setup screen replaces the containers.
- Setup screen: the Captain mark, "Set up Captain Engine", the resources, and "Use an existing engine". If Captain finds other engines, it lists them with a "Bring your data along" checkbox. After setup, that opens the Migration Assistant ([ADR 0009](../adr/0009-migration.md)).
- Starting screen: a moving bar and the last progress lines. Stopped screen: "Captain Engine is stopped" with Start. Failed screen: the error and "Try again", which reuses what the first start downloaded.
- Sidebar: the engine card shows the host state with Start or Stop. The brand line follows the host. (Replaced: the status bar shows the state, and the Diagnostics Engine card has Start, Stop, and Restart.)
- Settings: a Captain Engine card with the choice, status with Start, Stop, and Restart, CPUs, memory, and disk (applied on the next start; since 0037 a running engine shows Restart to apply, and the disk asks before it grows), the quit switch, "Bring data from another engine…", and "Reset Captain Engine…" with a confirmation.
- Menu bar: the status line comes from the host, with "Start Captain Engine" or "Stop Captain Engine".
- Quit: stops the engine first when the setting is on, waits up to 20 seconds, and keeps the window responsive. A start that runs at Quit is stopped whatever the setting, and also when the user chose another engine while it ran, because Captain cannot finish its later steps after it exits. During a snapshot step, Quit waits until the step ends (feature [0023](0023-snapshots.md)).
- Stop during a start: every step of the start runs under one cancel handle, `captain-host/src/cancel.rs`: `limactl start`, the read and the write of the daemon settings, the k3s version list and download with `curl`, and each `limactl shell` step of the k3s install. Stop kills the running command, and the start checks the handle before each next step, so it ends within seconds. Stop waits until the start has ended, with no time limit, and then stops the VM; the steps outside the handle (`limactl list`, `limactl --version`) give up after 30 seconds. The handle holds the child before it writes standard input, and it writes on its own thread, so a large `daemon.json` cannot block a kill. A killed `limactl shell` can leave `ssh` holding its pipes, so a cancelled command does not wait for them.
- Stop stops Docker first: `limactl shell captain sudo -n sh -c ...` runs `systemctl stop docker.socket docker.service` (and `k3s.service` first when k3s is installed), with a 60-second limit, then `limactl stop`. `dockerd` stops every container in parallel with its own stop signal and timeout, and waits up to the longest timeout plus 5 seconds. Because the daemon stops them, restart policies still bring `always` and `unless-stopped` containers back at the next start; `docker stop` would mark them as stopped by the user. If Docker does not stop in 60 seconds or the command fails, Captain logs a warning and stops the VM anyway. If `limactl stop` fails, Captain logs it and runs `limactl stop --force`. Every VM stop goes through this path: Stop, Restart, Restart to apply, Quit, and the stop before a snapshot or a restore. Reset deletes the VM, so it has nothing to stop.
- A start refuses while `.restore-backup` is in the instance folder, because an unfinished snapshot restore may have left missing or mixed files (feature [0023](0023-snapshots.md)).
- Automatic reconnect: when the connection fails (the event stream closes, a container list or `info` call fails, or a connect fails) and the chosen engine should answer, Captain connects again by itself. "Should answer" means Captain Engine runs, or the settings choose another engine. While Captain Engine is stopped, starts, or stops, the host model connects when it runs, so the workspace does not retry. The waits are 1, 2, 4, 8, and 16 seconds, then 30 seconds between attempts, until a connect succeeds, the user clicks Retry, or the user switches engines; a success resets them. After a working connection drops, the pages show "Reconnecting to the engine…" for 15 seconds, then the failure screen; a connect that never worked shows the failure at once and retries behind it. An automatic reconnect keeps the page, the focused project, and the selected container when it still exists. Code: `crates/captain-ui/src/workspace/auto_reconnect.rs`.
- One connection screen for every engine page (Containers, Project, Images, Volumes, Networks): "Reconnecting" during the quiet period, then the failure with Retry, "Restart Captain Engine" when Captain Engine runs, and "Captain keeps trying to reconnect by itself." The status bar engine segment and the rail's app icon turn the warning color while Captain reconnects and red after the quiet period, with a help sentence that says so.
- Status checks have limits: `limactl list`, `limactl --version`, and the daemon and k3s status reads in the VM give up after 30 seconds, so a hung `limactl` cannot freeze the status.

## Out of scope

- Bundling `limactl`, `docker`, and Compose in `Captain.app` (M9). Captain already looks for a bundled `limactl` first.
- A WSL2 host for Windows, and a rootless `dockerd` that Captain manages on Linux.
- A faster custom VM (ADR 0008, option B).
- Copying data between engines (M13, the Migration Assistant).

## Notes

- **`LIMA_HOME` is `~/.captain/lima`, not `~/Library/Application Support/Captain/lima`.** Lima puts Unix sockets (`ssh.sock.<random>`, `ha.sock`, `ga.sock`, and the forwarded `sock/docker.sock`) in the instance folder. On macOS a socket path must be shorter than 104 bytes. The Application Support path is long and has a space, and with a 32-character user name the longest socket path is 114 bytes. Lima avoids that folder for the same reason ([Lima internals](https://lima-vm.io/docs/dev/internals/)), and Rancher Desktop fails for user names of 21 or more characters because it uses it ([rancher-desktop#797](https://github.com/rancher-sandbox/rancher-desktop/issues/797)). With `~/.captain/lima` the longest path for a 32-character user name is 87 bytes. Captain checks the length and reports a clear error if a path would not fit.
- **Lima 2.2.0 or newer is required.** Lima 2.2.0 fixed a guest agent leak that silently killed port forwarding after many connections ([lima#5210](https://github.com/lima-vm/lima/issues/5210)). Captain reads `limactl --version` and shows "Lima 2.2.0 or newer is required" for an older one. The template sets `minimumLimaVersion: 2.2.0`.
- **The first start can hang waiting for SSH** ([lima#4517](https://github.com/lima-vm/lima/issues/4517), open). Captain gives `limactl start` a 30-minute timeout, streams progress, and offers "Try again", which reuses the created instance.
- **The Docker socket can die after the Mac sleeps** ([lima#5420](https://github.com/lima-vm/lima/issues/5420), open, planned for Lima 2.3; the same symptom in [rancher-desktop#9839](https://github.com/rancher-sandbox/rancher-desktop/issues/9839)). The VM keeps running but the socket refuses connections. When Captain Engine runs but does not answer, the error screen offers "Restart Captain Engine".
- **`limactl stop` gives the guest 30 seconds.** It sends SIGINT to the host agent, which asks the vz VM to stop (`RequestStop`, a power button press that `systemd-logind` turns into a poweroff) and waits 30 seconds; then it logs `vz timeout while waiting for stop status` and exits, which kills the VM, and `limactl stop` still exits 0 ([`Stop` in pkg/driver/vz/vz_driver_darwin.go](https://github.com/lima-vm/lima/blob/v2.2.0/pkg/driver/vz/vz_driver_darwin.go), [`startRoutinesAndWait` in pkg/hostagent/hostagent.go](https://github.com/lima-vm/lima/blob/v2.2.0/pkg/hostagent/hostagent.go), [pkg/instance/stop.go](https://github.com/lima-vm/lima/blob/v2.2.0/pkg/instance/stop.go)). Inside that window systemd stops `docker.service` (`KillMode=process`, `TimeoutStopSec` 90 seconds by default), and `dockerd` stops the containers ([`Daemon.Shutdown` in daemon/daemon.go](https://github.com/moby/moby/blob/docker-v29.8.2/daemon/daemon.go), [`shutdown-timeout`](https://docs.docker.com/reference/cli/dockerd/)). A guest that is slow to shut down, for example under memory pressure, never reaches that step, and PostgreSQL then starts with "database system was not properly shut down; automatic recovery in progress". Stopping Docker first, while the guest is healthy, avoids that. A daemon shutdown does not set `HasBeenManuallyStopped` ([daemon/kill.go](https://github.com/moby/moby/blob/docker-v29.8.2/daemon/kill.go)), so restart policies hold. With `live-restore` in `daemon.json`, `dockerd` leaves containers running when it stops, and the VM poweroff stops them instead.
- **Docker can restart while the VM runs.** `systemctl restart docker`, a `daemon.json` change, or a crash restarts `dockerd` without a change in the Lima status, so the host model sees no reason to reconnect. The daemon ends the `/events` stream when it stops ([Docker Engine API, `GET /events`](https://docs.docker.com/reference/api/engine/version/v1.51/), [`docker system events`](https://docs.docker.com/reference/cli/docker/system/events/)), and without `live-restore` it stops the running containers, whose restart policies bring them back ([Live restore](https://docs.docker.com/engine/daemon/live-restore/)). Captain treats the closed stream as a drop and reconnects with backoff. A chaos test found the old behavior: Captain showed "the engine closed the event stream" until the user clicked Retry on the Containers page.
- **Bind mount ownership:** with virtiofs, files in bind mounts can show the wrong uid and gid inside containers ([lima#4053](https://github.com/lima-vm/lima/issues/4053), open).
- **`/tmp/lima`** is mounted because the design asks for it. Lima no longer mounts it by default and advises against it on shared computers ([lima#3648](https://github.com/lima-vm/lima/issues/3648)).
- **Rosetta** needs `softwareupdate --install-rosetta` on a Mac without it, or Lima can wait at "Installing rosetta..." ([Lima multi-arch docs](https://lima-vm.io/docs/config/multi-arch/), [lima#1202](https://github.com/lima-vm/lima/issues/1202)).
- **curl limits.** `--connect-timeout` covers only the connection ([curl docs](https://curl.se/docs/manpage.html#--connect-timeout)). A k3s download also fails when it moves slower than 1000 bytes per second for 60 seconds (`--speed-limit`, `--speed-time`), and a text download stops after 120 seconds (`--max-time`).
- `limactl list --json` prints one JSON object per line. Captain reads all instances and picks `captain`, like Colima does, because `limactl list captain` fails when the instance does not exist. Statuses: `Running`, `Stopped`, `Broken` (with `errors`), and `Installing`.
- The template uses `base: template:_images/ubuntu-lts`, so `limactl` must find its `share/lima/templates` folder. M9 must bundle it next to `limactl`.

## Development setup

1. Install Lima 2.2 or newer: `brew install lima`.
2. Run Captain. With no Captain Engine yet, it shows the setup screen.
3. Captain's VM lives in `~/.captain/lima`. It never touches `~/.lima` or `~/.colima`.
4. To look at the VM by hand, set `LIMA_HOME` first: `LIMA_HOME=~/.captain/lima limactl list`.
5. The live test creates, starts, checks, stops, and deletes a VM named `captain-agent-test` in a temporary folder: `cargo test -p captain-host --test live -- --ignored --nocapture`.

## Verification

1. With Lima installed and no `~/.captain/lima/captain`, Captain opens on the setup screen and connects to nothing.
2. "Set up Captain Engine" shows the moving bar and progress lines, then the containers page of an empty engine.
3. `DOCKER_HOST=unix://$HOME/.captain/lima/captain/sock/docker.sock docker run --rm hello-world` works.
4. Stop in the sidebar shows "Captain Engine is stopped"; Start brings the containers back. A PostgreSQL container logs "database system was shut down at" after the next start, not "automatic recovery in progress".
5. Quit with the switch on stops the VM (`LIMA_HOME=~/.captain/lima limactl list` shows `Stopped`); with it off, the VM keeps running.
6. The next launch starts a stopped engine by itself.
7. A resources change in Settings applies after Restart (`limactl list` shows the new CPUs and memory).
8. "Reset Captain Engine…" asks first, then deletes the VM, and the setup screen returns.
9. "Use an existing engine" connects to Rancher Desktop or another engine, as before.
10. The menu bar status line and the Start or Stop item follow the engine.
11. With a Project page open, run `LIMA_HOME=~/.captain/lima limactl shell captain sudo systemctl restart docker`. The page shows "Reconnecting to the engine…" and the status bar engine segment turns the warning color. Within a few seconds the page returns on the same project, with the same container selected, and the segment turns green. `captain.log` has "reconnected to the engine".
12. Run `sudo systemctl stop docker.socket docker.service` in the VM. After 15 seconds the page shows the failure with Retry and "Restart Captain Engine", and the segment turns red. `sudo systemctl start docker` brings the page back without a click.

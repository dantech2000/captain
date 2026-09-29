# Feature 0013: Captain Engine

- Milestone: M12
- Status: In progress
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
- Sidebar: the engine card shows the host state with Start or Stop. The brand line follows the host.
- Settings: a Captain Engine card with the choice, status with Start, Stop, and Restart, CPUs, memory, and disk (applied on the next start), the quit switch, "Bring data from another engine…", and "Reset Captain Engine…" with a confirmation.
- Menu bar: the status line comes from the host, with "Start Captain Engine" or "Stop Captain Engine".
- Quit: stops the engine first when the setting is on, waits up to 20 seconds, and keeps the window responsive.

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
- **Bind mount ownership:** with virtiofs, files in bind mounts can show the wrong uid and gid inside containers ([lima#4053](https://github.com/lima-vm/lima/issues/4053), open).
- **`/tmp/lima`** is mounted because the design asks for it. Lima no longer mounts it by default and advises against it on shared computers ([lima#3648](https://github.com/lima-vm/lima/issues/3648)).
- **Rosetta** needs `softwareupdate --install-rosetta` on a Mac without it, or Lima can wait at "Installing rosetta..." ([Lima multi-arch docs](https://lima-vm.io/docs/config/multi-arch/), [lima#1202](https://github.com/lima-vm/lima/issues/1202)).
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
4. Stop in the sidebar shows "Captain Engine is stopped"; Start brings the containers back.
5. Quit with the switch on stops the VM (`LIMA_HOME=~/.captain/lima limactl list` shows `Stopped`); with it off, the VM keeps running.
6. The next launch starts a stopped engine by itself.
7. A resources change in Settings applies after Restart (`limactl list` shows the new CPUs and memory).
8. "Reset Captain Engine…" asks first, then deletes the VM, and the setup screen returns.
9. "Use an existing engine" connects to Rancher Desktop or another engine, as before.
10. The menu bar status line and the Start or Stop item follow the engine.

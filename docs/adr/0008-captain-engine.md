# ADR 0008: Captain Engine on Lima, behind an EngineHost interface

- Status: Accepted
- Date: 2026-09-29

## Context

Captain only works while another product runs an engine for it. On this machine that is Rancher Desktop. When Rancher is closed, Captain shows "engine not reachable" and can do nothing. Rancher also supplies the `docker` and Compose CLIs that Captain's Compose actions use (`~/.rd/bin`).

Captain is meant to be its own product. It must start, stop, and configure its own engine.

On macOS, every Docker product runs `dockerd` in a Linux VM. We compared three ways to get one:

| Option | Summary | Verdict |
|--------|---------|---------|
| A. Lima | Captain drives `limactl` with its own template. Lima uses Apple's Virtualization.framework (`vmType: vz`), virtiofs, and Rosetta. Colima and Rancher Desktop are built the same way. | Chosen |
| B. Own VM in Rust | Drive Virtualization.framework directly (the OrbStack approach). Fastest boot, full control. | Later, maybe |
| C. Control other engines | Start and stop Rancher, Colima, or Docker Desktop through their CLIs. | Rejected: Captain stays a layer on top |

Option B needs a Linux kernel and root filesystem we build and patch forever, an in-VM agent for port forwarding, DNS and time sync, and `unsafe` Objective-C calls. It is months of work to reach what Lima does today. A boot time of 10 to 20 seconds is acceptable for now.

## Decision

### The interface

`captain-core` defines an `EngineHost` trait. The UI talks only to it, so Option B can replace Lima later without UI changes (the same pattern as the terminal emulator, ADR 0007):

- `status() -> HostStatus`: `NotInstalled`, `Stopped`, `Starting`, `Running`, `Stopping`, or `Failed(message)`.
- `start() -> stream of progress lines`: the first start downloads the guest image and installs Docker, which takes minutes, so the UI shows progress.
- `stop()`.
- `endpoint() -> Endpoint`: the Docker socket to connect to once it runs.
- `resources()` and `set_resources(cpus, memory, disk)`: a change applies on the next start.
- `reset()`: delete the VM and its data, after a confirmation in the UI.

### The Lima host

- **Instance:** one Lima instance named `captain`.
- **Isolation:** `LIMA_HOME` is `~/.captain/lima`, so Captain never sees or changes the user's own Lima VMs.
  - Not `~/Library/Application Support/Captain/`: Lima keeps several Unix sockets in the instance folder (`ha.sock`, `ssh.sock`, `serialv.sock`, the forwarded `sock/docker.sock`). On macOS a socket path must be shorter than 104 characters. [Lima's internals docs](https://lima-vm.io/docs/dev/internals/) say this is why Lima itself uses `~/.lima`. The Application Support path is long and has a space, so a long user name would break it.
- **Lima version:** at least 2.2.0. Older versions leak a socket per forwarded connection until port forwarding silently stops ([lima#5210](https://github.com/lima-vm/lima/issues/5210), fixed by [PR #5216](https://github.com/lima-vm/lima/pull/5216), released in v2.2.0). Captain checks `limactl --version` and the template sets `minimumLimaVersion: 2.2.0`.
- **First start can time out:** provisioning with the `docker-rootful` recipe sometimes waits a long time for SSH ([lima#4517](https://github.com/lima-vm/lima/issues/4517), open). The setup screen shows progress, allows a retry, and never treats one failed start as fatal.
- **Template:** Captain writes its own template, based on Lima's `docker-rootful` template:
  - Ubuntu LTS guest, `vmType: vz`, `mountType: virtiofs`, Rosetta on (on Apple Silicon).
  - Rootful `dockerd` with the containerd snapshotter.
  - The guest socket `/var/run/docker.sock` is forwarded to `<instance dir>/sock/docker.sock`. That path is the endpoint.
  - The home folder is mounted writable, so bind mounts from projects work as they do in Docker Desktop.
- **Default resources:** half the host CPUs (minimum 2, maximum 8), 4 GiB of memory or a quarter of host memory if that is larger (maximum 16 GiB), and a 64 GiB sparse disk.
- **Finding `limactl`:** first the copy inside `Captain.app` (added in M9), then `PATH` and Homebrew. If neither exists, `status()` is `NotInstalled`, and the UI explains how to install Lima during development.

### How Captain uses it

- **Settings** gains an engine choice: **Captain Engine** (the default once it exists) or **Other engine** (today's behavior: `DOCKER_HOST`, contexts, known sockets, or a custom endpoint). Other engines are connected to, never controlled.
- **Launch:** if Captain Engine is the choice and it is stopped, Captain starts it and shows progress where the containers would be.
- **Quit:** a setting "Stop the engine when Captain quits", on by default, like Docker Desktop. Closing the window does not quit (the menu bar keeps Captain running, ADR 0006).
- **Sidebar and menu bar:** the engine card and the tray menu get Start and Stop.
- **First launch:** if no Captain Engine exists yet, Captain shows a setup screen: "Set up Captain Engine" (with an offer to bring data from engines it detects, see ADR 0009) or "Use an existing engine".
- **CLI:** the `docker` and Compose CLIs still come from the host until M9 bundles them. The Compose features (ADR 0005) keep their "CLI not found" state for that case.

### Other platforms

- **Linux:** no VM is needed. A `SystemHost` reports the system `dockerd` and does not control it. Rootless `dockerd` managed by Captain can come later.
- **Windows:** a WSL2 host is a separate, later implementation of the same trait.

## Consequences

- Captain works without Rancher, Docker Desktop, or OrbStack.
- The first start takes a few minutes (image download and Docker install). Later starts take 10 to 20 seconds.
- Development needs Lima installed (`brew install lima`). The release app will carry its own `limactl`, `docker`, and Compose binaries (M9). All are Apache-2.0 licensed.
- Containers in the old engine do not appear in Captain Engine by themselves. ADR 0009 covers moving them.
- A faster custom VM (Option B) stays possible behind `EngineHost`.

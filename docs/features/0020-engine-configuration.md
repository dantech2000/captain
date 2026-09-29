# Feature 0020: Engine configuration

- Milestone: M18
- Status: Implemented; the live test passes on a test VM. The card needs a check by hand in the app.
- Design: [ADR 0008](../adr/0008-captain-engine.md), [feature 0013](0013-captain-engine.md)

## Goal

Let the user configure the Docker daemon in Captain Engine from Settings, as Rancher Desktop's Container Engine preferences do. This covers three of Rancher Desktop's most requested issues:

- [rancher-desktop#1330](https://github.com/rancher-sandbox/rancher-desktop/issues/1330): configure `/etc/docker/daemon.json` from the GUI.
- [rancher-desktop#719](https://github.com/rancher-sandbox/rancher-desktop/issues/719): configure registry mirrors.
- [rancher-desktop#2851](https://github.com/rancher-sandbox/rancher-desktop/issues/2851): expose the Docker daemon over TCP.

## In scope

- A **Docker daemon** card in Settings, below the Captain Engine card. It shows while Captain Engine is the chosen engine and Captain controls it (macOS).
  - **Registry mirrors**: one URL per line (commas also work). Each needs `https://` or `http://`, because `dockerd` rejects a mirror without a scheme.
  - **Insecure registries**: one `host:port` or CIDR per line, without a scheme.
  - **Custom daemon.json**: a JSON object with any other `dockerd` keys, for example `{"log-level": "warn"}`. Save checks that the text is valid JSON and that it is an object.
  - **Expose the Docker API on TCP**: a switch (off by default) and a port (default 2375, 1024 to 65535). The note warns that the API has no TLS, so any program on the Mac can control Docker. The endpoint is `tcp://127.0.0.1:<port>`.
  - **Save** checks all fields and saves them, or shows the first problem in red.
  - **Restart to apply**, with a Restart button, while the saved settings differ from the ones the running engine uses.
- Settings file: `engine_daemon` with `registry_mirrors`, `insecure_registries`, `custom`, `tcp`, and `tcp_port`. Old files load with the defaults.
- The merge rules and the field checks are pure functions in `captain_core::daemon`, with unit tests.

## Out of scope

- **Allowed images** (Rancher Desktop's image allow list). Rancher Desktop enforces it with a proxy in front of the registries. `daemon.json` has no key for it, so it is not simple. It moves to a later milestone.
- TLS for the TCP API. A user can add `tlsverify`, `tlscacert`, `tlscert`, and `tlskey` in the custom JSON and copy the files into the VM. Captain does not manage certificates.
- Listening on an address other than `127.0.0.1` on the Mac.
- Live reload (`SIGHUP`). Only some keys reload ([docs](https://docs.docker.com/reference/cli/dockerd/#configuration-reload-behavior)), so every change waits for a restart.
- Linux: Captain does not control the system `dockerd` (`SystemHost`), so the card stays hidden.

## Merge rules

Captain writes `/etc/docker/daemon.json` from the saved settings:

1. Start with the custom JSON object.
2. Set `registry-mirrors` and `insecure-registries` from their fields. A field that is empty leaves the key out.
3. Merge `features` key by key: the user's features stay, and `cdi` and `containerd-snapshotter` are always `true`.

Captain manages some keys. Save rejects them in the custom JSON with a message that says why, and the merge drops them if the settings file has them anyway:

| Key | Why |
|-----|-----|
| `hosts` | The TCP switch sets the listeners. `hosts` in `daemon.json` together with `-H` in the systemd unit stops `dockerd` from starting ([moby#22339](https://github.com/moby/moby/issues/22339), [Docker docs](https://docs.docker.com/engine/daemon/remote-access/)). |
| `containerd` | The systemd unit passes `--containerd`. The same key in both places is also a conflict. |
| `registry-mirrors`, `insecure-registries` | Their own fields set them. |
| `features.cdi`, `features.containerd-snapshotter` | The Lima template sets them on every boot. The containerd image store holds the images; turning it off hides them. |

## How a change reaches the VM

A change applies on the next start of Captain Engine. After `limactl start` finishes and the Docker socket exists, Captain:

1. Reads `/etc/docker/daemon.json` and the TCP drop-in over `limactl shell`.
2. Compares them with the saved settings as JSON values, so formatting does not matter. If they match, it does nothing.
3. Otherwise it sends the new `daemon.json` on standard input to a script run with `sudo` in the guest. The script:
   1. Runs `dockerd --validate --config-file=<new file>`. If `dockerd` rejects it, nothing changes and the start fails with `dockerd`'s message.
   2. Keeps a copy of the old files, installs the new `daemon.json`, and writes or removes the drop-in `/etc/systemd/system/docker.service.d/captain-tcp.conf`.
   3. Runs `systemctl daemon-reload` and `systemctl restart docker`. If the restart fails, it puts the old files back and restarts Docker again.
   4. From the copy until the end, an `EXIT` trap puts both old files back and runs `systemctl daemon-reload` on any other failure, for example a full disk while it writes the drop-in ([POSIX trap](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap02.html#trap)). A hangup or termination exits through the same trap. The trap is cleared when the new settings run.

Why this and not the Lima template:

- The template only applies when Captain creates the VM. Changing it later needs `limactl edit` of `provision`, and a provision script runs while `docker.service` may already be up, so it would need its own restart anyway.
- `limactl shell` works on every existing VM, whatever template version created it.
- `dockerd --validate` and the roll back mean a bad key never leaves the engine without Docker.
- The files stay in place across reboots. Lima's `yq` provision step sets the two `features` keys on every boot; Captain's file already has them, so the step changes nothing.

### The TCP socket

- `dockerd` gets `-H tcp://127.0.0.1:<port>` in a systemd drop-in, next to the unit's own `-H fd://`, the way Docker's [remote access docs](https://docs.docker.com/engine/daemon/remote-access/) show. The drop-in copies the unit's `ExecStart` line and adds the flag, so other flags stay as Docker ships them. `daemon.json` never has `hosts`, so the conflict cannot happen.
- In the guest, `dockerd` listens on `127.0.0.1` only. Lima's default port forwarding rule forwards a guest port on `127.0.0.1` to the same port on the host's `127.0.0.1` ([Lima port forwarding](https://lima-vm.io/docs/config/port/), and the `portForwards` notes in Lima's [default.yaml](https://github.com/lima-vm/lima/blob/master/templates/default.yaml): "Lima internally appends this fallback rule at the end"). So Captain does not edit `lima.yaml`, and the port is never on the network.
- Docker says remote access without TLS "is not recommended". Anyone who can reach the port has root in the VM, so the card warns before the switch.

### Restart to apply

- `EngineHost` gains `set_daemon` (the settings for the next start) and `running_daemon` (what the running engine uses, when known).
- After Captain applies the settings in a start, it records them as running. If the engine already ran when Captain launched, the first status check reads the guest files once. A stopped engine forgets them.
- The card shows **Restart to apply** while the engine runs and the running settings differ from the saved ones.

## Notes

- `dockerd --validate` exists since Docker 23.0. Captain Engine installs the current Docker from get.docker.com.
- `limactl shell` quotes its arguments and passes standard input through (checked with Lima 2.2.0).
- A failed apply fails the start with a notification. The engine keeps running with the old settings, and the next status check shows it as running.

## Verification

Unit tests cover the merge rules, the checks for each field, the guest file parser, and the script arguments.

Live, on a test VM (`captain-agent-daemon` in `~/.clo/lima`, a short path for Lima's sockets):

```sh
PATH=~/.rd/bin:$PATH cargo test -p captain-host --test live_daemon -- --ignored --nocapture
```

It starts the VM through `LimaHost` with a registry mirror, `{"log-level": "warn"}`, and TCP on port 23750. It checks that `docker info` lists the mirror and that `DOCKER_HOST=tcp://127.0.0.1:23750 docker version` works from the Mac. A second start with the same settings shows no "Applying" line. It then adds an unknown key: the start fails with `dockerd`'s message, the running settings stay the old ones, and Docker still answers. Last, it reverts to the defaults: the mirror is gone and the TCP port is closed. It deletes the VM and `~/.clo` at the end, unless `CAPTAIN_KEEP_VM` is set. It passed in 106 seconds with a new VM, and in 60 seconds with an existing one. The apply code needed no fix.

By hand (on a test VM, not a VM with data you need):

1. Open Settings. With Captain Engine chosen, the Docker daemon card shows below the Captain Engine card.
2. Enter `mirror.gcr.io` as a registry mirror and click Save. A red message asks for `https://`.
3. Enter `https://mirror.gcr.io`, and `{"log-level": "warn"}` as custom JSON. Click Save. **Restart to apply** appears.
4. Enter `{"hosts": []}` or `[1]` as custom JSON. Save shows why it is not accepted.
5. Click Restart. The progress shows "Applying the Docker daemon settings." Then `LIMA_HOME=~/.captain/lima limactl shell captain cat /etc/docker/daemon.json` shows the mirror, the log level, and both features. **Restart to apply** is gone.
6. `docker info` against Captain Engine lists the mirror under Registry Mirrors.
7. Turn on the TCP switch and restart. `DOCKER_HOST=tcp://127.0.0.1:2375 docker version` works, and `lsof -iTCP:2375 -sTCP:LISTEN` shows only `127.0.0.1`.
8. Enter `{"bogus": 1}` and restart. The start fails with `dockerd`'s message, and Docker still runs with the previous settings.
9. Restart again without changes. No "Applying" line shows.

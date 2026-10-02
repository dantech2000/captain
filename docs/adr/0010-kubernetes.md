# ADR 0010: Kubernetes as k3s inside Captain Engine

- Status: Accepted
- Date: 2026-09-29

## Context

M19 adds an optional Kubernetes cluster. Rancher Desktop and Docker Desktop both ship one, and people who move from them expect `kubectl` to work after they flip a switch.

Captain Engine is one Lima VM named `captain` in `~/.captain/lima` (ADR 0008). It runs rootful `dockerd` from Lima's `docker-rootful` template. The home folder is mounted writable, so the guest can read files in `~/.captain`.

The cluster must meet four needs:

1. An image built with `docker build` runs in a pod without a push or an import.
2. The user picks a Kubernetes version, and Captain downloads it once.
3. The cluster costs nothing while it is off.
4. Reset clears the cluster without deleting the user's images.

We compared three ways to run it:

| Option | How it works | Images built with Docker | Cost | Verdict |
|--------|--------------|--------------------------|------|---------|
| A. k3s inside the Captain Engine VM | Captain installs the k3s binary in the guest and runs it as a service, with `--docker`. Rancher Desktop does the same inside its own Lima VM. | Visible at once. k3s talks to the same `dockerd`. | No second VM. k3s adds its own memory to the one VM. | Chosen |
| B. Lima's `k3s` or `k8s` template as a second VM | Captain creates a second Lima instance from `template:k3s` or `template:k8s`. | Not visible. The cluster has its own containerd, so every image needs `docker save` and an import. | A second kernel, disk, and memory reservation. Two VMs boot. | Rejected |
| C. kind or k3d as containers on the Docker engine | Each Kubernetes node is a Docker container that runs its own containerd. Docker Desktop offers kind as one of its two cluster types. | Not visible. Each image needs `kind load docker-image` or `k3d image import`. | No second VM, but a nested containerd with its own image copies. | Rejected |

Sources for the table:

- Rancher Desktop starts k3s with `--docker` when the engine is moby, or points it at k3s's own containerd socket otherwise ([service-k3s.initd](https://github.com/rancher-sandbox/rancher-desktop/blob/e4c91fe/pkg/rancher-desktop/assets/scripts/service-k3s.initd)).
- k3s 1.24 and newer bundle cri-dockerd, which is what `--docker` uses ([k3s advanced options](https://docs.k3s.io/advanced#using-docker-as-the-container-runtime)).
- Lima's `k3s` template installs k3s with its bundled containerd, turns mounts off, and copies the kubeconfig to `<instance>/copied-from-guest/kubeconfig.yaml` ([templates/k3s.yaml](https://github.com/lima-vm/lima/blob/v2.2.0/templates/k3s.yaml)).
- kind needs `kind load docker-image` to put a local image into the cluster ([kind quick start](https://kind.sigs.k8s.io/docs/user/quick-start/#loading-an-image-into-your-cluster)). k3d has `k3d image import` for the same job ([k3d docs](https://k3d.io/stable/usage/commands/k3d_image_import/)).
- Docker Desktop offers kubeadm or kind, and names its context `docker-desktop` ([Docker Desktop Kubernetes](https://docs.docker.com/desktop/features/kubernetes/)).

Option C has real strengths: several clusters, several nodes, and nothing new inside the VM. But the image import step on every build is the first thing Rancher and Docker Desktop users would notice, and it breaks need 1. Option B breaks needs 1 and 3. Option A meets all four.

## Decision

Captain runs k3s inside the Captain Engine VM, with `dockerd` as its container runtime. Kubernetes is off by default.

### Versions, download, and cache

- **The version list** comes from the GitHub releases API for `k3s-io/k3s`, and the channel labels come from the k3s channel server, `https://update.k3s.io/v1-release/channels`. The channel server returns JSON with a `latest` version per channel, for example `stable` and `latest`. Rancher Desktop reads the same two sources ([k3sHelper.ts](https://github.com/rancher-sandbox/rancher-desktop/blob/e4c91fe/pkg/rancher-desktop/backend/k3sHelper.ts)).
- **The default** is the `stable` channel's version at the time the user turns Kubernetes on. Captain saves the exact version in `settings.json`, so the cluster never upgrades by itself.
- **The picker** in Settings lists stable releases only, newest first, with the channel label next to the version (`v1.36.4+k3s1 (stable)`). Captain hides versions below a floor. The floor skips k3s releases whose bundled cri-dockerd cannot talk to Docker 25 or newer ([k3s#9279](https://github.com/k3s-io/k3s/issues/9279)). Rancher Desktop keeps the same list of bad ranges in [backendHelper.ts](https://github.com/rancher-sandbox/rancher-desktop/blob/e4c91fe/pkg/rancher-desktop/backend/backendHelper.ts). Docker Engine 29 refuses clients older than API 1.44 ([Docker 29 release notes](https://docs.docker.com/engine/release-notes/29/)), so the floor must stay above every k3s that shipped an older cri-dockerd. We set the floor after a live test, and raise it when a Docker upgrade needs it.
- **Offline:** Captain caches the version list in `~/.captain/cache/k3s-versions.json`. With no network, the picker shows cached and downloaded versions only.
- **Download:** for a version, Captain downloads three assets from `https://github.com/k3s-io/k3s/releases/download/<version>/`: the binary (`k3s` or `k3s-arm64`), the air-gap images (`k3s-airgap-images-<arch>.tar.zst`), and `sha256sum-<arch>.txt`. It checks the SHA-256 sums, then renames the temp folder to `~/.captain/cache/k3s/<version>/`. These are the names Rancher Desktop downloads. The air-gap images let k3s start without pulling its system images ([k3s air-gap install](https://docs.k3s.io/installation/airgap)).
- **Install in the guest:** the home mount already exposes `~/.captain/cache`. Captain copies the binary to `/usr/local/bin/k3s` in the guest and loads the air-gap images with `docker load`. No download happens inside the VM.
- **Upgrade and downgrade** follow Rancher's rules ([Kubernetes preferences](https://docs.rancherdesktop.io/ui/preferences/kubernetes)). An upgrade keeps workloads. A downgrade needs a reset of the cluster state, and Captain asks first. Images stay in both cases.

### The k3s service

Captain writes a systemd unit in the guest. Its command line is:

```
k3s server --docker --https-listen-port <port> --write-kubeconfig-mode 644 [--disable traefik]
```

- **Port:** 6443 by default, changeable in Settings. Lima's default rule forwards any guest TCP port on `127.0.0.1` or `0.0.0.0` to the same port on the host's `127.0.0.1` ([Lima default template](https://github.com/lima-vm/lima/blob/v2.2.0/templates/default.yaml)), so the API server appears on `https://127.0.0.1:6443` with no extra rule. If the port is in use on the host, Settings says so before it restarts the cluster.
- **Traefik:** on by default, like Rancher Desktop and k3s. A switch turns it off with `--disable traefik`, which frees ports 80 and 443 ([k3s networking services](https://docs.k3s.io/networking/networking-services)). Turning Traefik off on an existing cluster does not remove the installed chart ([k3s#5103](https://github.com/k3s-io/k3s/issues/5103)). Captain deletes the `traefik` Helm chart through the API after start, as Rancher does. Ports 80 and 443 on the host need a Lima rule with `hostIP: 0.0.0.0`, because the default `127.0.0.1` rule cannot bind a privileged port (the comment on `portForwards` in the Lima default template). That rule exposes the ports to the local network, so Captain adds it only when the user turns on "Expose Traefik on ports 80 and 443".
- **Start and stop:** the unit is enabled only while Kubernetes is on. Turning Kubernetes on or off restarts k3s, not the VM. Stopping Captain Engine stops k3s with it.
- **Runtime limits:** cri-dockerd does not support every CRI feature. For example, image volume mounts fail with `--docker` ([k3s#14452](https://github.com/k3s-io/k3s/issues/14452)). The Kubernetes settings page links to a short list of known gaps.

### Kubeconfig and context

- After k3s writes `/etc/rancher/k3s/k3s.yaml`, Captain reads it from the guest and renames the cluster, user, and context to `captain`. (Later renamed to `captain-desktop`; see the last change below.) Rancher Desktop uses `rancher-desktop` for all three ([k3sHelper.ts, `updateKubeconfig`](https://github.com/rancher-sandbox/rancher-desktop/blob/e4c91fe/pkg/rancher-desktop/backend/k3sHelper.ts)).
- The server URL is `https://127.0.0.1:<port>`.
- Captain merges the entry into the first file in `KUBECONFIG` that already has a `captain` context, or else into `~/.kube/config`. It replaces only the `captain` entries and keeps every other cluster, user, and context.
- Captain sets `current-context: captain` only if the file has no current context. The menu bar lists all contexts and switches between them (the M19 menu bar item).
- Reset and "Turn off Kubernetes" leave the context in place. "Reset Captain Engine" removes it.

### Images built with Docker

With `--docker`, kubelet asks `dockerd` for images through cri-dockerd, so `docker build -t app:dev .` followed by a pod with `image: app:dev` and `imagePullPolicy: IfNotPresent` works. There is no import step and no second image store. This is the main reason for option A.

### Reset

"Reset Kubernetes…" asks first. Then Captain stops k3s and deletes its state in the guest: `/var/lib/kubelet`, `/var/lib/rancher/k3s/data`, `/var/lib/rancher/k3s/server`, `/var/lib/rancher/k3s/storage`, `/etc/rancher/k3s`, and `/run/k3s`. This is the list Rancher Desktop deletes in `deleteKubeState`. Docker images and volumes stay, and so do containers that the user started with `docker` or Compose. Captain removes the pod containers that k3s created (the ones with an `io.kubernetes.pod.namespace` label).

### Pod containers in the container list

With `--docker`, every pod container is also a Docker container, so it appears in Captain's list. Captain hides containers with the `io.kubernetes.pod.namespace` label by default and shows them behind a "Show Kubernetes containers" filter.

### Resource cost

A k3s server needs at least 2 CPUs and 2 GB of memory ([k3s requirements](https://docs.k3s.io/installation/requirements)). Captain Engine's default memory is 4 GiB (ADR 0008). When the user turns Kubernetes on with less than 6 GiB for the engine, Settings shows the memory it recommends and offers to raise it. The change applies on the next engine start, as today. With Kubernetes off, k3s does not run and costs only the cached files on disk.

### Port Forwarding page

A new page lists Kubernetes Services, grouped by namespace, with their ports. Each port has Forward. Forward opens a listener on `127.0.0.1` with a port the user picks or a free one, and relays each connection to a pod behind the Service through the Kubernetes port-forward API. Rancher Desktop does the same with a local server and the client library's `PortForward` ([kube/client.ts](https://github.com/rancher-sandbox/rancher-desktop/blob/e4c91fe/pkg/rancher-desktop/backend/kube/client.ts)). In Rust, `kube` provides this with `Api::portforward` and `Portforwarder` ([docs.rs](https://docs.rs/kube/latest/kube/api/struct.Portforwarder.html)). Only ports above 1024 are offered. Forwards end when Captain quits, and the page lists the active ones.

The page shows only while Kubernetes runs. Container ports that Docker publishes need no page: Lima forwards them already.

### Where the code goes

- `captain-host` gets a `KubernetesHost` trait next to `EngineHost`, with `status`, `enable(version, options)`, `disable`, `reset`, and `kubeconfig`. The Lima host runs the guest commands with `limactl shell`.
- The version list, download, and cache live in `captain-core`, with unit tests on saved channel and release JSON.
- `kube` is a new dependency of the crate that owns the Port Forwarding page. It runs on the tokio runtime, like `bollard` (ADR 0002).

## Consequences

- `docker build` then `kubectl apply` works with no import step, as in Rancher Desktop with the moby engine.
- Captain has one VM. Turning Kubernetes on adds k3s's memory use to that VM and no VM boot time. We measure the real number on the live test before we pick the 6 GiB threshold for good.
- Captain depends on cri-dockerd inside k3s. A Docker upgrade in the guest can break older k3s versions, as k3s#9279 did. The version floor and a live test for each Docker upgrade guard against this.
- Some Kubernetes features that need a CRI runtime other than Docker do not work. Users who need them can run kind on top of Captain Engine themselves.
- Only one cluster and one node. Multi-node clusters are out of scope.
- The cluster lives on the Captain Engine disk, so snapshots (ADR 0012) include it.
- The `captain` context in `~/.kube/config` outlives a disabled cluster until the user resets Captain Engine.

## Changes during implementation

M19 follows this decision, with these changes. See [feature 0024](../features/0024-kubernetes.md).

- **Version floor.** It is `v1.29.2+k3s1`, the first release with the k3s#9279 fix. The live test ran `v1.36.4+k3s1` on Docker 29.8.1.
- **Where the code went.** The `KubernetesHost` trait lives in `captain-core`, next to `EngineHost`, which returns it from `EngineHost::kubernetes()`. It has `status`, `versions`, `enable`, `disable`, `reset`, and `kubeconfig`. `captain-core` parses the channel and release lists and owns the cache format, but it has no network code: `curl` and the SHA-256 check are in `captain-host`.
- **kube-rs** is in a new crate, `captain-kube`, not in the UI crate. It implements a `PortForwarding` trait from `captain-core`, so `captain-ui` stays free of kube types, as it is of bollard types.
- **Kubeconfig target.** Captain merges into the first `KUBECONFIG` file with a `captain` context, else the first `KUBECONFIG` file, else `~/.kube/config`. This is kubectl's own rule. It also means a test with its own `KUBECONFIG` never writes to `~/.kube/config`. Each write first copies the file to `<file>.captain-backup`.
- **`socat` in the VM.** cri-dockerd forwards ports with `nsenter` and `socat`, and the Ubuntu image has no `socat`. The first install runs `apt-get install socat`. This is the only download inside the VM.
- **Ready means usable.** Captain waits until systemd reports the unit active, `/readyz` answers, and the `default` service account exists. Before that, a first pod fails with "serviceaccount default not found".
- **Turning Kubernetes off** also stops the pod containers. They are Docker containers and outlive k3s.
- **Downgrade.** The host refuses a lower version. The Settings card asks, saves the version, and resets. The CLI saves the version and tells the user to run `captain kubernetes reset`.
- **Reset Captain Engine** removes the `captain` context in the app's host model, not in `LimaHost`, so live tests that delete a test VM never touch a kubeconfig.
- **Context name, 2026-10-01.** The cluster, user, and context are now `captain-desktop`, like `docker-desktop` and `rancher-desktop`. The name `captain` read as Captain's Docker context, which is `captain-engine`. Each merge removes Captain's old `captain` entries from every kubeconfig file, and a current context of `captain` becomes `captain-desktop`. Captain knows its old entries by the `captain` cluster: a server at `https://127.0.0.1:<port>` and the same certificate authority as the new `captain-desktop` cluster. A `captain` context for another cluster stays. Reset Captain Engine removes both names on the same rule. The backup works as before. A Reset Kubernetes or a port change before the first merge gives a new authority or server, so the old entries stay and the user deletes them with `kubectl config delete-context captain`.
- **Not built yet:** hiding pod containers in the container list, and the "Expose Traefik on ports 80 and 443" rule. The port check runs only while k3s is off, because a running cluster holds its port through Lima.

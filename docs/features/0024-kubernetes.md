# Feature 0024: Kubernetes

- Milestone: M19
- Status: Implemented; the live test passes on a test VM. The Settings card, the Port Forwarding page, and the menu bar need a check by hand in the app.
- Parity: Rancher Desktop's [Kubernetes preferences](https://docs.rancherdesktop.io/ui/preferences/kubernetes) and [Port Forwarding](https://docs.rancherdesktop.io/ui/port-forwarding) page
- Decision: [ADR 0010](../adr/0010-kubernetes.md)

## Goal

A user turns on a switch and gets a one-node Kubernetes cluster in Captain Engine. `kubectl` works with the `captain` context. An image built with `docker build` runs in a pod without a push or an import. A Service port can be forwarded to a port on the Mac.

## In scope

- **k3s in Captain Engine**, started with `--docker`, so kubelet uses the engine's `dockerd` through k3s's bundled cri-dockerd. Kubernetes is off by default. When it is off, k3s does not run.
- **Versions.** The list comes from the k3s channel server (`https://update.k3s.io/v1-release/channels`) and the first three pages of the GitHub releases of `k3s-io/k3s`. It shows stable releases at or above `v1.29.2+k3s1`, newest first, with the channels each one leads: `v1.36.4+k3s1 (stable, v1.36)`. The list is cached in `~/.captain/cache/k3s-versions.json`. Offline, the cache and the downloaded versions answer. An engine start that looks up the stable version fetches the list under the start's cancel handle, so Stop ends it.
- **Version tags.** A tag must look like `v1.36.4+k3s1` or `v1.37.1-rc2+k3s1`: digits, and a prerelease of letters, digits, and dots. The tag names a cache folder, so anything else is refused.
- **Download.** Captain downloads `k3s` or `k3s-arm64`, `k3s-airgap-images-<arch>.tar.zst`, and `sha256sum-<arch>.txt` with `curl` into a temporary folder. It checks each SHA-256, then renames the folder to `~/.captain/cache/k3s/<version>/`. A version downloads once.
- **Install.** Over `limactl shell`, Captain copies the binary to `/usr/local/bin/k3s`, loads the air-gap images with `docker load` once per version, and writes the unit `/etc/systemd/system/k3s.service`:

  ```
  k3s server --docker --https-listen-port <port> --write-kubeconfig-mode 644 [--disable traefik]
  ```

  It restarts k3s only when the binary or the unit changed. It then waits until systemd reports the unit active, `/readyz` answers, and the `default` service account exists, so a first `kubectl apply` works.
- **Start and stop with the engine.** Each engine start installs and starts k3s when Kubernetes is on, or disables the unit when it is off. A k3s failure does not fail the engine start: the progress log shows a warning, and the Kubernetes card shows the state. Stopping the engine stops k3s with it.
- **Kubeconfig.** Captain reads `/etc/rancher/k3s/k3s.yaml`, renames the cluster, user, and context to `captain`, and sets the server to `https://127.0.0.1:<port>`. It writes `~/.captain/kubeconfig` with only that context. It merges the entries into the user's kubeconfig only when Kubernetes starts: into the first file in `KUBECONFIG` that has a `captain` context, or else the first file in `KUBECONFIG`, or else `~/.kube/config`. Only the `captain` entries change. The current context becomes `captain` only when no file in the list sets one. kubectl takes the current context from the first file that sets it ([Organizing cluster access](https://kubernetes.io/docs/concepts/configuration/organize-cluster-access-kubeconfig/#merging-kubeconfig-files)), so with `KUBECONFIG=first:second`, a `current-context` in `second` stays in effect when Captain writes to `first`. Before each write, Captain copies the file to `<file>.captain-backup`. A symlinked kubeconfig, for example one that chezmoi manages, stays a link: Captain writes to its target, and the backup sits next to the link. The new file is created with mode 0600 before the client key goes in.
- **Settings: a Kubernetes card** below the Docker daemon card, while Captain controls Captain Engine:

  | Row | What it does |
  |-----|--------------|
  | Enable Kubernetes | A switch and the state (Off, Starting, Running, Failed). Turning it on saves the stable version when none is saved, so the cluster never upgrades by itself. |
  | Kubernetes version | A picker. An upgrade keeps the workloads. A lower version than the running one asks first, then resets the cluster. |
  | Kubernetes port | The API port, 6443 by default. Apply saves it. If the port is in use on the Mac, Apply fails with a message. |
  | Enable Traefik | On by default. Off adds `--disable traefik` and deletes the `traefik` and `traefik-crd` Helm charts, which the flag leaves ([k3s#5103](https://github.com/k3s-io/k3s/issues/5103)). |
  | More memory recommended | Shows when the engine has less than 6 GiB. **Use 6 GB** changes the engine memory for its next start. |
  | Apply | Applies the settings to the running engine now, and shows each progress line. Otherwise the next engine start applies them. |
  | Reset | **Reset Kubernetes…** asks first, then deletes the cluster state and starts an empty cluster. |

- **Reset Kubernetes** stops k3s, removes the pod containers (label `io.kubernetes.pod.namespace`), unmounts and deletes `/var/lib/kubelet`, `/var/lib/rancher/k3s/{data,server,storage}`, `/etc/rancher/k3s`, and `/run/k3s` (the list Rancher Desktop deletes), and starts k3s again if it is on. Images, volumes, and containers started with Docker stay.
- **Turning Kubernetes off** disables the unit and stops the pod containers, because they are Docker containers and outlive k3s. The state stays for the next start. The `captain` context stays in the kubeconfig. **Reset Captain Engine** in the app removes it.
- **A Port Forwarding page.** Its sidebar entry sits above Diagnostics and shows only while the cluster runs. The page lists the Services with TCP ports, one card per namespace. **Forward** asks for a local port above 1024, or takes a free one. The row then shows `127.0.0.1:<port>` and **Stop**. Each connection finds a running pod behind the Service, resolves `targetPort` (a number, a container port name, or unset), and relays through the Kubernetes port-forward API with kube-rs `Api::portforward`. Forwards end when Captain quits. Each connection reads Captain's kubeconfig and builds a new client when the file changed, so a forward keeps working after Reset Kubernetes writes new certificates. A failed accept, such as too many open files, waits 250 ms before the next.
- **Pod containers in the container list.** With `--docker`, cri-dockerd runs each pod container as a Docker container named `k8s_<container>_<pod>_<namespace>_…` with `io.kubernetes.*` labels. The Containers page hides the containers with the `io.kubernetes.pod.namespace` label by default, as Rancher Desktop's Containers page does ([docs](https://docs.rancherdesktop.io/ui/containers)). While the engine has any, the header shows a **Show Kubernetes containers** checkbox. When it is on, the list shows one card per namespace ("Kubernetes namespace · 3 of 4 running") after the Compose projects and before the loose containers. The header counts, the Running tile, and the sidebar count follow the checkbox. The checkbox is not saved; each launch starts with it off.
- **Menu bar: a Kubernetes Contexts submenu** with every context in the user's kubeconfig files and a check mark on the current one. Picking one makes it current in the first file that sets a current context, or the first file, as `kubectl config use-context` does. The tray reads the files every 5 seconds, so changes from `kubectl` show up.
- **CLI:**

  | Command | What it does |
  |---------|--------------|
  | `captain kubernetes enable [--version V] [--port P] [--traefik true\|false]` | Saves the settings with Kubernetes on, and starts k3s if the engine runs. The default version is the saved one, or the stable one. |
  | `captain kubernetes disable` | Saves Kubernetes off, and stops k3s if the engine runs. |
  | `captain kubernetes status [--json]` | The settings, the state, the running version, the kubeconfig path, and whether the `captain` context is in the user's kubeconfig. |
  | `captain kubernetes reset [--yes]` | Asks, then resets the cluster. The engine must run. |

  `enable` and `disable` refuse while the app runs, like `captain set`. They change only the options the command names, on the settings read inside the settings lock, so two commands keep each other's change. A downgrade through `enable --version` saves the version and fails with a hint to run `captain kubernetes reset`.

## Out of scope

- A namespace picker like Rancher Desktop's. The namespace cards fold instead.
- Rancher's "Expose Traefik on ports 80 and 443" rule with `hostIP: 0.0.0.0`.
- Privileged local ports (1024 and below) on the Port Forwarding page.
- UDP Service ports, pod ports without a Service, and forwards that survive a restart of Captain.
- containerd as the runtime, the Spin operator, several clusters, and several nodes.
- Kubernetes on Linux and Windows hosts. `EngineHost::kubernetes()` is `None` there, and the card and page do not show.

## Notes

- **cri-dockerd needs `socat`.** It forwards a pod port by running `nsenter … socat - TCP4:localhost:<port>` ([streaming_others.go](https://github.com/Mirantis/cri-dockerd/blob/master/streaming/streaming_others.go)). The Ubuntu image has no `socat`, so the first install runs `apt-get install socat` in the VM. Without it, every forward closes at once.
- **Version floor.** k3s releases before February 2024 bundle a cri-dockerd that cannot read images from Docker 25 and newer ([k3s#9279](https://github.com/k3s-io/k3s/issues/9279)); the fix shipped in v1.29.2, v1.28.7, and v1.27.11. Captain offers `v1.29.2+k3s1` and newer. The live test ran `v1.36.4+k3s1` on Docker 29.8.1.
- **Why `--docker`.** k3s 1.24 and newer bundle cri-dockerd for it ([k3s advanced options](https://docs.k3s.io/advanced#using-docker-as-the-container-runtime)). Rancher Desktop starts k3s the same way with the moby engine ([service-k3s.initd](https://github.com/rancher-sandbox/rancher-desktop/blob/e4c91fe/pkg/rancher-desktop/assets/scripts/service-k3s.initd)).
- **Air-gap images.** With `--docker`, k3s does not import `agent/images` itself, so Captain loads the tarball with `docker load` ([k3s air-gap install](https://docs.k3s.io/installation/airgap)).
- **API port.** Lima's default rule forwards a guest TCP port on `0.0.0.0` to the same port on the host's `127.0.0.1` ([Lima default template](https://github.com/lima-vm/lima/blob/v2.2.0/templates/default.yaml)), so no extra rule is needed. Captain reads the port of the installed unit. It checks the host port when k3s is not running or the port changes, because a running cluster holds its own port through Lima. After the API answers in the VM, Captain reads `k3s.yaml`, saves its certificate authority in `~/.captain/kubernetes-ca.pem`, and waits up to 60 seconds for `https://127.0.0.1:<port>/ping` on the Mac to answer `pong` with `curl --cacert` on that file. Another program or another local k3s on the port has a certificate from another authority, so the check fails and the kubeconfig never points at it. k3s's serving certificate is valid for `127.0.0.1`, the server address in its own `k3s.yaml` ([k3s server options](https://docs.k3s.io/cli/server)). k3s serves `/ping` without a login ([router.go](https://github.com/k3s-io/k3s/blob/master/pkg/server/handlers/router.go)). The port must be 1 to 65535; the CLI and the host both check it.
- **Engine lock.** Apply, Reset, and the CLI commands take the engine lock with the note `kubernetes`, and need a running engine that no start or stop is changing.
- **YAML.** Kubeconfig files are parsed and written with `serde-saphyr` as JSON values, so only the `captain` entries change. Comments in a merged file are lost, as with `kubectl config`. The backup keeps the original.
- **kube-rs.** `kube` 4.2 with `rustls-tls`, `ring`, and `ws`, and `k8s-openapi` 0.28 with `latest` ([docs.rs Portforwarder](https://docs.rs/kube/latest/kube/api/struct.Portforwarder.html), [pod_portforward_bind example](https://github.com/kube-rs/kube/blob/main/examples/pod_portforward_bind.rs)). It lives in the new `captain-kube` crate on its own tokio runtime, as bollard lives in `captain-docker` (ADR 0002).
- **Code layout.** `Container::kube_namespace` comes from the label in `captain-docker/src/mapping/container.rs`. `captain_core::store::GroupKey` orders the cards (project, namespace, standalone), and `ContainerStore::groups(filter, kubernetes)` hides or groups the pod containers. `captain_core::kubernetes`: settings, versions, channel and release parsing, asset names, checksums, the kubeconfig merge and files, the `KubernetesHost` and `PortForwarding` traits, and target port resolution. `captain-host/src/k3s`: `curl`, the checked download, and the version list with its cache. `captain-host/src/lima/kubernetes`: the guest scripts, the install steps, and `LimaKubernetes`. `captain-kube`: the kube client, Services, pod lookup, and `KubeForwarder`. `captain-ui/src/kubernetes` and `settings/kube*.rs`: the model and the card. `captain-ui/src/port_forwarding`: the page. `captain-app/src/tray/contexts.rs`: the submenu. `captain-cli/src/commands/kubernetes`: the four commands.

## Verification

Automated (`cargo test -p captain-core -p captain-host -p captain-cli -p captain-app`):

- Version order and parsing, the channel and release lists with the floor, channel labels, and cached versions.
- Checksum lookup, the download URL, file hashing, and a complete cache folder that needs no download.
- The `captain` kubeconfig from `k3s.yaml`, the merge that keeps other entries and the current context, removal, the target file in a `KUBECONFIG` list, the backup, and `use-context`.
- A current context set in a later `KUBECONFIG` file stays current after the install.
- The install arguments, the unit state parsing, `targetPort` resolution, the local port rule, the CLI arguments, and the contexts submenu.
- Pod containers: the namespace label mapping, hidden by default, and one card per namespace when shown.

Live: with a kubeconfig of its own, never the real one:

```sh
mkdir -p ~/.ck8 && PATH=~/.rd/bin:$PATH KUBECONFIG=~/.ck8/kubeconfig \
  cargo test -p captain-kube --test live_kubernetes -- --ignored --nocapture
```

It creates `captain-agent-k8s` in `~/.ck8/lima`, starts it with Kubernetes on (the stable version), checks the state and the `captain` context, builds `captain-agent-k8s:test` with `docker build`, runs it in a pod with `imagePullPolicy: Never` behind a Service with a named `targetPort`, forwards the Service port, reads the page through it, stops the forward, resets the cluster, and checks that the Service is gone. It deletes the VM and `~/.ck8` at the end. It passed in 97 seconds; the engine and k3s came up in about 60 seconds.

By hand in the app:

1. Open **Settings**. The Kubernetes card shows **Enable Kubernetes** off.
2. Turn it on. The version picker shows the stable version. Click **Apply**. The note shows the download and the start, then the state is Running.
3. Run `kubectl --context captain get nodes`. A `~/.kube/config.captain-backup` exists if the file existed before.
4. Build an image with `docker build -t app:dev .` and run it with `kubectl run app --image app:dev --image-pull-policy Never`.
5. Expose it with `kubectl expose pod app --port 80 --target-port <port>`. Open **Port Forwarding**, click **Forward** on `app:80`, and leave the port empty. Open `http://127.0.0.1:<port>`.
6. In the menu bar, open **Kubernetes Contexts** and pick another context. `kubectl config current-context` shows it.
7. Click **Reset Kubernetes…** and confirm. The pod is gone and the image stays.
8. Open **Containers**. No `k8s_` containers show. Turn on **Show Kubernetes containers**. A `kube-system` card and a `default` card (with `app`) appear, and the counts grow.
9. Turn Kubernetes off and click **Apply**. The state is Off and the Port Forwarding entry leaves the sidebar.

On a test VM (`captain-agent-daemon` in `~/.clo/lima`, with a temporary `KUBECONFIG`), `captain kubernetes enable` started `v1.36.4+k3s1`, and `docker ps -a` listed ten `k8s_*` containers, each with `io.kubernetes.pod.namespace=kube-system`.

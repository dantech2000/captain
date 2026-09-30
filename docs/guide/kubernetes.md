# Kubernetes

Captain can run a one-node Kubernetes cluster ([k3s](https://k3s.io)) inside Captain Engine. The cluster uses the engine's Docker, so an image that you build with `docker build` runs in a pod without a push.

Kubernetes needs Captain Engine, so it works only on macOS. It is off by default. While it is off, k3s does not run and uses no memory.

## Turn it on

1. Open Settings.
2. Turn on the **Kubernetes** switch.
3. Wait until the status bar shows "Kubernetes on". The first start downloads k3s, which takes a minute or two.

You can also use the **Kubernetes** row in the menu bar popover, or run `captain kubernetes enable` with Captain closed.

Kubernetes needs about 2 GB of memory of its own. Give Captain Engine at least 6 GB in Settings.

When you turn Kubernetes on the first time, Captain saves the current stable k3s version. The cluster never upgrades by itself.

## Use kubectl

Captain adds a context named `captain` to `~/.kube/config`, and keeps your other contexts. Before each change, it saves a copy as `~/.kube/config.captain-backup`.

```sh
kubectl --context captain get nodes
```

If no other context is current, `captain` becomes the current context. To switch contexts, right-click the menu bar icon and open **Kubernetes Contexts**, or run `kubectl config use-context captain`.

Captain does not ship `kubectl` or `helm`. Install them, for example with `brew install kubectl helm`.

Run a local image without a push:

```sh
docker build -t app:dev .
kubectl --context captain run app --image app:dev --image-pull-policy Never
```

## Pods in the container list

Each pod container is also a Docker container. Captain hides them by default. Turn on **Show Kubernetes containers** on the Containers page to show one entry per namespace in the sidebar and in the list.

## Port Forwarding

While the cluster runs, the sidebar has a **Port Forwarding** button. The page lists each service with TCP ports, grouped by namespace.

1. Click **Forward** on a service port.
2. Enter a **Local port** above 1024, or leave it empty for any free port.
3. Click **Forward**. The row shows `127.0.0.1:PORT` and **Stop**.

Forwards end when Captain quits. The palette command `forward svc/<name> [port]` does the same. See [The command palette](command-palette.md#forward).

## Other options

The k3s version, the API port, and Traefik are options in the [settings file](settings.md):

| Option | Default | What it does |
|--------|---------|--------------|
| `kubernetes.version` | the stable version when you first turned it on | The k3s version, such as `v1.36.4+k3s1`. An upgrade keeps the workloads. A downgrade needs a reset. |
| `kubernetes.port` | `6443` | The port of the Kubernetes API on `127.0.0.1`. |
| `kubernetes.traefik` | `true` | The Traefik ingress controller on ports 80 and 443. Set `false` to free those ports. |

Changes apply at the next engine start. The [settings reference](../reference/settings.md) lists every option.

## Reset the cluster

A reset deletes the workloads and the cluster state, then starts an empty cluster. Images, volumes, and containers that you started with Docker stay.

```sh
captain kubernetes reset
```

The engine must run. The command asks first.

## Turn it off

Turn off the **Kubernetes** switch. Captain stops k3s and its pod containers. The cluster keeps its state for the next time. The `captain` context stays in your kubeconfig.

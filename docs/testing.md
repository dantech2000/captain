# Manual test guide

This guide covers what the automated tests cannot: the app on a real Captain Engine. Each test lists its steps and the result to expect. Run them on macOS with a debug build.

## Before you start

1. Build the app: `scripts/bundle-macos.sh debug`.
2. Quit any open Captain. Then open `target/debug/Captain.app`.
3. Make sure Captain Engine shows **Running** in the sidebar.
4. Open **Diagnostics**. All checks must show **Passed** or **Not applicable**.

Some tests stop or restart the engine. Containers with a restart policy (for example `unless-stopped`) come back by themselves. Do these tests when a short outage is acceptable.

If a result is different from the expected result, open **Diagnostics > Show logs** and keep `captain.log` for the bug report.

## 1. Docker daemon settings (M18)

Restarts Docker in the engine once.

1. Open **Settings > Docker daemon**.
2. In **Registry mirrors**, type `https://mirror.gcr.io`. Click **Save**.
   - Expect: the **Restart to apply** row appears.
3. Click **Restart**.
   - Expect: the status shows "Applying the Docker daemon settings", then **Running**.
4. In a terminal, run `docker info --format '{{.RegistryConfig.Mirrors}}'` with `DOCKER_HOST` set by `captain docker-env`.
   - Expect: `[https://mirror.gcr.io/]`.
5. In **Custom daemon.json**, type `{"hosts": ["x"]}`. Click **Save**.
   - Expect: an error that says Captain manages `hosts`. Nothing changes.
6. Remove the mirror. Click **Save**, then **Restart**.
   - Expect: `docker info` shows no mirrors.

## 2. Snapshots (M16)

Stops the engine twice. The restore replaces the engine disk.

1. Open **Snapshots**. Click **Create snapshot…**, keep the default name, and click **Create**.
   - Expect: the engine stops, the snapshot row appears with a size, and the engine starts again.
2. Create a test volume: `docker volume create captain-test-snapshot`.
3. On the snapshot row, click **Restore…**. Keep **Save the current state first** on. Click **Restore**.
   - Expect: a "Before restore …" snapshot appears. The engine starts again.
4. Run `docker volume ls`.
   - Expect: `captain-test-snapshot` is gone. Your other volumes and containers are as they were when you took the snapshot.
5. Click **Edit…** on a snapshot, change the name and description, and save.
   - Expect: the row shows the new name.
6. Delete both test snapshots with **Delete…**.
   - Expect: the rows go away and `~/.captain/snapshots` gets smaller.
7. Start a restore, then close the main window (the menu bar icon stays).
   - Expect: the engine still starts again after the restore. Reopen the window from the menu bar to check.

## 3. Kubernetes (M19)

Installs k3s in the engine (about 70 MB download) and adds a `captain` context to your kubeconfig. Captain backs up the kubeconfig first.

1. Open **Settings > Kubernetes**. Turn on **Enable**, keep the default version, and click **Apply**.
   - Expect: progress lines, then the card shows the cluster as running. This can take a few minutes on the first run.
2. Run `kubectl --context captain get nodes`.
   - Expect: one node, `Ready`.
3. Build and run a pod from a local image:
   ```
   printf 'FROM nginx:alpine\n' | docker build -t captain-k8s-test:dev -
   kubectl --context captain run web --image=captain-k8s-test:dev --image-pull-policy=Never --port=80
   kubectl --context captain expose pod web --port=80
   ```
   - Expect: the pod runs without a pull.
4. Open **Port Forwarding**. Forward service `web` port 80 to local port 18080.
   - Expect: `curl -s localhost:18080` returns the nginx page.
5. On **Containers**, check that pod containers are hidden. Turn on **Show Kubernetes containers**.
   - Expect: a card per namespace.
6. In the menu bar, open **Kubernetes Contexts** and switch to another context and back.
   - Expect: `kubectl config current-context` follows.
7. Click **Stop** on the engine while k3s starts (turn Kubernetes off and on, then Apply, then Stop at once).
   - Expect: the stop finishes within seconds.
8. Clean up: delete the pod and service, then turn Kubernetes off and Apply.

## 4. Administrative access (M14)

Asks for your macOS password.

1. Open **Settings > Administrative access**. Click **Link to Captain Engine…** and enter your password.
   - Expect: the card says `/var/run/docker.sock` points at Captain Engine.
2. Run `docker -H unix:///var/run/docker.sock ps`.
   - Expect: the same containers as in Captain.
3. Click **Remove link…** and enter your password.
   - Expect: the card says the socket does not exist.

## 5. Extensions (M21)

1. Open **Extensions**. Install `docker/disk-usage-extension:0.2.9` and confirm the trust dialog.
2. Click **Open**.
   - Expect: a window with the Disk Usage chart. Do not click **Reclaim space**; it prunes images and volumes.
3. Click **Remove…**.
   - Expect: the row goes away, and the image and backend are removed.

## 6. Contexts and remote hosts (M7)

1. Open **Settings > Switch engine**.
   - Expect: one row per `docker context ls` entry, with the current one marked.
2. If you have a host with SSH key login and Docker, enter `ssh://user@host` in **Custom endpoint** and click **Use this engine**.
   - Expect: Captain shows that engine's containers. Quitting Captain leaves no `ssh -nNT` process.

## 7. Command line (M20)

Use `target/debug/Captain.app/Contents/Resources/bin/captain`.

1. Run `captain status`, `captain info`, and `captain docker-env`.
   - Expect: the engine state, versions, and an `export DOCKER_HOST=…` line.
2. With the app open, run `captain set cpus 4`.
   - Expect: it refuses because Captain is running.
3. Quit the app. Run `captain stop`, then `captain start`.
   - Expect: the engine stops and starts again.

## 8. Quit behavior

1. With **Stop the engine when Captain quits** on, quit Captain.
   - Expect: the engine stops. The next launch starts it again.
2. Start a snapshot, then press Cmd-Q.
   - Expect: Captain exits only after the snapshot step ends.

## Last run

2026-09-29, commit c656372, macOS, on the real Captain Engine. Run through the `captain` CLI, which drives the same host code as the app. The UI steps were not run, because the terminal had no Accessibility permission.

| Test | Result |
|------|--------|
| 1. Daemon settings: mirror, custom key, TCP 23750, revert | Pass. `docker info` showed the mirror; `tcp://127.0.0.1:23750` answered; revert removed the drop-in. About 12 s outage per restart. |
| 2. Snapshots: create, restore, rename, delete | Pass. Create and restore took about 10 s each. The APFS clone used no extra disk. The test volume was gone after restore. |
| 3. Kubernetes: enable, local-image pod, port forward, reset, disable | Pass. k3s v1.36.4 ran in 40 s. The `captain` context was added, the current context was kept, and the kubeconfig backup was written. |
| 3.7 Stop during a k3s start | Not run (needs the app). |
| 2.7, 8.2 Close the window or quit during a snapshot | Not run (needs the app). |
| 4. Administrative access | Not run (needs your password). |
| 5. Extensions | Pass earlier the same day, in the app. |
| UI pages: Snapshots, Kubernetes card, Port Forwarding, contexts menu | Not run. |

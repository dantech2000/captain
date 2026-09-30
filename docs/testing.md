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

## 9. The v3 window: projects, tasks, and the status bar (M22–M24)

Uses a throwaway Compose project. Save this as `compose.yaml` in an empty folder:

```yaml
name: captain-test-v3
services:
  web:
    image: nginx:alpine
    ports: ["18089:80"]
    environment:
      API_URL: http://worker:9000
  worker:
    image: alpine:3.21
    command: ["sh", "-c", "sleep 5; tail /dev/zero"]
    mem_limit: 64m
    restart: always
x-captain:
  tasks:
    hello:
      service: web
      command: echo hello from a Captain task
```

1. Run `docker compose up -d` in that folder. Open the project in the sidebar.
   - Expect: a red dot on the project, and a red **worker** card: "Restarting", "Exit 137 · out of memory", and **Raise memory to 512 MB**.
2. Hover controls. Expect a sentence in the status bar for each. With nothing hovered, the bar shows "…worker-1 exited again at …, after N restarts".
3. Click **hello** in Tasks.
   - Expect: the output under the button, "hello exited with 0", and a toast.
4. Wait for the worker to crash again.
   - Expect: a red line in the project log, "worker exited 137 (out of memory) · …".
5. Click **Raise memory to 512 MB**. Run `docker inspect -f '{{.HostConfig.Memory}}' captain-test-v3-worker-1`.
   - Expect: `536870912`.
6. Click the panel button at the top right of the details panel.
   - Expect: the panel hides, and a rail with a button stays at the right edge.

## 10. Map and staged changes (M28)

1. On the test project, click **Map**.
   - Expect: `:18089` on the left, **web** and **worker** in one network lane, and a line from web to worker (from `API_URL`).
2. Click **Stage** on the worker, then **Apply**.
   - Expect: a toast, and `docker inspect` shows the new limit.

## 11. Menu bar popover and floating log (M26)

1. Left-click the wheel in the menu bar while the worker crash-loops.
   - Expect: a warning card, "…worker-1 keeps restarting: out of memory at …", with **Raise**, **Show logs**, and **Stop**. It stays while the worker runs between crashes.
2. Click **Float logs**, or the window button in the details panel header.
   - Expect: a small window with that container's log that stays on top.
3. Check the Dock icon (if the Dock shows).
   - Expect: a red badge with the number of problems.
4. Right-click the wheel.
   - Expect: the plain menu.

## 12. ⌘K commands (M27)

1. Press ⌘K and type `restart po`.
   - Expect: rows for the matching containers with the typed part in bold. Tab completes.
2. Type `logs <container> --since 10m --errors` and press Return.
   - Expect: the Logs tab with a Since chip and the Error filter.

## 13. Storage (M25)

1. Open Storage from the sidebar Disk card.
   - Expect: the disk bar, the largest items with who uses them, and the five cleanup groups. Unused volumes are unchecked.
2. Click **Review…**.
   - Expect: a list of every item to remove. Cancel it unless you want the cleanup.

Clean up: `docker compose down` in the test folder.

## Last run

2026-09-29, commit c656372, macOS, on the real Captain Engine. Tests 1–3 ran first through the `captain` CLI (the same host code as the app), then the UI steps in the app.

| Test | Result |
|------|--------|
| 1. Daemon settings: mirror, custom key, TCP 23750, revert | Pass (CLI). `docker info` showed the mirror; `tcp://127.0.0.1:23750` answered; revert removed the drop-in. About 12 s outage per restart. The form checks passed earlier in the app. |
| 2.1–2.6 Snapshots page: create, edit, restore with save first, delete | Pass. Create took under 3 s plus the engine restart. The APFS clone used no extra disk. The test volume was gone after restore, and a "Before restore" snapshot was saved. |
| 2.7 Close the window during a restore | Pass. The restore finished, the engine started again, and the page was normal after reopening. |
| 3.1–3.2 Kubernetes card: enable and apply | Pass. The card showed Running once the CA-checked API answered. The `captain` context was added and the current context was kept. |
| 3.3–3.4 Local-image pod and Port Forwarding page | Pass. The pod ran without a pull; forwarding `web:80` to 18080 served it; Stop closed the port. |
| 3.5 Show Kubernetes containers | Pass. Hidden by default; the toggle shows one card per namespace. |
| 3.6 Kubernetes Contexts menu | Pass. Switching to `captain` and back changed `kubectl config current-context`. |
| 3.7 Stop during the k3s step of a start | Pass. The engine stopped 4 s after Stop. |
| 3.8 Turn Kubernetes off | Pass. No pod containers left; engine memory back to about 57 MB. |
| 4. Administrative access | Pass. Link: after the password prompt, `/var/run/docker.sock` is a root-owned link to Captain Engine's socket, `docker -H unix:///var/run/docker.sock ps` reaches it, and the card shows the link. Remove link: after a second password prompt the link is gone and the card offers Link to Captain Engine… again. |
| 5. Extensions | Pass, in the app. |
| 6. Contexts list | Pass after a fix: "No engines found" no longer shows above listed contexts. |
| 7. Command line | Pass. |
| 8.1 Quit stops the engine; the next launch starts it | Pass. The engine was up 9 s after launch. |
| 8.2 Quit during a snapshot | Pass. Captain waited for the snapshot step, then quit. |
| 9. v3 window (test project) | Pass, 2026-09-30: the red worker card, the task output, the out-of-memory marker, Raise memory (536870912), and the collapsible panel. The status bar first said "then exited" during a crash loop; now "exited again at …, after N restarts". |
| 10. Map and staged changes | Pass: the talks-to line, Stage, and Apply (1 GB). The failing node's text overlapped; fixed. |
| 11. Popover and floating log | Pass after a fix: the warning card came and went during the crash loop, and Float logs opened the selected container. A crash tracker on the event stream fixed both. The Dock badge was not checked. |
| 12. ⌘K commands | Pass for completions. |
| 13. Storage | Pass for the page and the Disk card. No cleanup was run. |

Notes for the UI/UX pass:

- The sidebar's bottom entries and the engine card's Start button move when the engine stops or starts.
- The Memory tile and the menu bar count include Kubernetes containers while they are hidden.
- Kubernetes containers show raw `k8s_POD_…` names; the pod and container names would read better.

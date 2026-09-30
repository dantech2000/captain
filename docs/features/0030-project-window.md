# Feature 0030: Project-first window

- Milestone: M24
- Status: Implemented. It needs a check by hand in the app.
- Design: the `Manifest` screen of the v3 canvas. See [0027](0027-v3-interface.md).

## Goal

The main window opens on what the user works on: a project. The sidebar lists Compose projects, Kubernetes namespaces, and loose containers. A project page answers "what runs, where do I open it, and why did it crash" on one screen.

## In scope

- **Sidebar.**
  - The engine header on top: the engine glyph, "Captain Engine" (or "Engine"), and its state with CPUs and memory.
  - The search button.
  - "Projects", with an "All containers" link on the same line. One entry per Compose project: a stack glyph in the project's label color, the name, "Compose · N services", a running summary with a state dot, and the published ports as chips.
  - One entry per Kubernetes namespace that has containers, only while "Show Kubernetes containers" is on: a cluster glyph and "Kubernetes · N pods".
  - "Loose containers": the containers in no project and no namespace.
  - A spacer, then a row of icon buttons for the pages of all resources: Images, Volumes, Networks, Snapshots, Extensions, Port Forwarding (while the cluster runs), Diagnostics (with the failure badge), and Settings. It replaces the list of pages.
  - The Disk card above the icon row (feature 0031).
  - The engine card at the bottom: CPU and memory gauges, and with Captain Engine a Start, Set up, or Stop button. It has no title, because the engine header names the engine and its state. It keeps its height when the engine stops or starts, so the entries above it and its button do not move.
- **Project page** (`Page::Project`, for the sidebar entry in `Workspace::focus`). It works for all three kinds of entries. The header buttons, the Open row, and tasks need a Compose project.
  1. **Header.** The working folder and Compose file in mono, the name in large type, then Open folder, Terminal, Down, and a primary Restart project (Up while nothing runs).
  2. **Open row.** One pill per published port: the service and `localhost:PORT`. A click opens `http://localhost:PORT`. Well-known ports of databases and brokers (5432, 3306, 6379, 27017, 9092, and a few more) copy the address instead, and the help says so.
  3. **Service cards**, three columns. Each card: the container glyph in the state color, the service name, the image in mono, a state pill, a note (uptime, health, or the exit reason), CPU with a sparkline and memory from the stats board, a port link, and Logs and Shell buttons. Context actions: Resume for a paused container; "Raise memory to N MB" after an out-of-memory kill. A click on a card opens the inspector; a second click closes it.
     - A container that restarts shows "Exit 137 · out of memory · 3 times in 2 min". The exit code and the out-of-memory flag come from `inspect` (`State.ExitCode`, `State.OOMKilled`). The count is the `die` events of the last two minutes.
     - "Raise memory" doubles `HostConfig.Memory`, to at least 512 MB, with `POST /containers/{id}/update` ([Engine API: ContainerUpdate](https://docs.docker.com/reference/api/engine/version/v1.47/#tag/Container/operation/ContainerUpdate)). It also sets `MemorySwap` to twice the new limit, as `docker run` does when only `--memory` is given, so the engine does not refuse a limit above the old swap limit. The change lasts until Compose recreates the container; the help says so.
  4. **Tasks card**, from `x-captain.tasks` in the Compose file. With no tasks, a dashed card shows how to add one.
  5. **Project log.** All services in one list, ordered by time, with a colored service tag. It follows new lines and keeps its lines across restarts. When a service exits, a red divider says so: "worker exited 137 (out of memory) · 12:07:11".
- **Tasks format.** A top-level extension field ([Compose extensions](https://docs.docker.com/reference/compose-file/extension/)):

  ```yaml
  x-captain:
    tasks:
      migrate:
        service: api
        command: npm run migrate
      seed:
        service: api
        command: ["node", "scripts/seed.js"]
  ```

  `service` is required. A string `command` runs with `sh -c`; a list runs as it is. Captain reads the tasks with `docker compose -p NAME -f FILE... config --format json`, which keeps `x-` fields ([compose config](https://docs.docker.com/reference/cli/docker/compose/config/)). A task runs with `docker compose exec -T SERVICE ...` ([compose exec](https://docs.docker.com/reference/cli/docker/compose/exec/)). The card shows the exit code and the last lines of output. An entry without a service or a command shows as a problem line on the card.
- **Engine API.**
  - `ContainerApi::logs_with(id, LogOptions { tail, since })`, for the project log and later for M27 ([Engine API: ContainerLogs](https://docs.docker.com/reference/api/engine/version/v1.47/#tag/Container/operation/ContainerLogs), `since` in Unix seconds).
  - `ContainerApi::update_memory(id, bytes)`.
  - `ProjectRunner::tasks(project)` and `ProjectRunner::run_task(project, task)`.
  - `EngineEvent` gets the time, the container name, and the exit code from the event's actor attributes ([docker events](https://docs.docker.com/reference/cli/docker/system/events/)).
  - `ContainerDetail` gets the exit code, the out-of-memory flag, and the memory limit.
- **Fixes from testing.**
  - The Memory tile and the menu bar count leave out Kubernetes containers while they are hidden.
  - Kubernetes containers show as `pod/container`, read from the cri-dockerd name `k8s_<container>_<pod>_<namespace>_<uid>_<attempt>`. The pod's sandbox (`k8s_POD_…`) shows as `pod (sandbox)`.

## Out of scope

- Opening the user's editor. Captain cannot know it; "Open folder" opens the folder with the system's default app (`open`, `xdg-open`, or Explorer).
- `DOCKER_HOST` in the new Terminal window. `open -a Terminal DIR` cannot pass environment variables, so the shell uses the user's docker context. The help says so.
- Output that streams while a task runs. The card shows the output when the task ends.
- Shortcuts (⌘O, ⌘T, ⌘⇧R) from the design. They come with the command grammar (M27).

## Notes

- The project log opens one log stream per running container, with the last 100 lines. When a container starts again, Captain opens a new stream with `since` set to the time of the `start` event, so the list keeps its old lines and does not repeat them. The list keeps 2000 entries.
- Lines are ordered by their Docker timestamp in seconds. Lines of the same second keep the order they arrived in.
- An `oom` event before a `die` marks the exit as out of memory. Exit 137 alone could be any `SIGKILL`.
- Down removes the containers, so the project would drop out of the sidebar. The page keeps the last known project, so its header still offers Up.
- The inspector opens next to the cards only after a click, so the cards keep three columns while it is closed. With it open, they use two.

## Verification

1. Run `cargo test -p captain-core`. Check that the `project_task`, `kube_name`, `port_link`, and `project_log` tests pass.
2. Start a project with a published web port and a database port. Check its sidebar entry: name, "Compose · N services", the running summary, and the port chips.
3. Click the entry. Check the header, the Open row, and one card per service. Click the web pill; check that the browser opens. Click the database pill; check that the address is on the clipboard.
4. Add the `x-captain.tasks` example above to the Compose file. Reopen the project. Run a task; check the exit code and output on the card.
5. Run `docker update --memory 64m --memory-swap 128m` on a service, then make it allocate more memory. Check the red divider in the log and the "Exit 137 · out of memory" note. Click "Raise memory to 512 MB"; check `docker inspect` shows the new limit.
6. Restart the project. Check that the log keeps the old lines and adds new ones below a divider.
7. Turn Kubernetes on and show its containers. Check the namespace entry and the `pod/container` names. Hide them; check that the Memory tile and the menu bar count drop.
8. Stop and start the engine. Check that the sidebar entries and the Start button do not move, and that "Captain Engine" shows only in the header.

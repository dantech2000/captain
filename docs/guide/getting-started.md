# Getting started

This page covers macOS. See [the guide index](README.md#platforms) for Linux and Windows.

## Install Captain

Captain needs macOS 15 or later.

### From a release

1. Open the [releases page](https://github.com/dantech2000/captain/releases).
2. Download `Captain-<version>-arm64.dmg`. Releases have an Apple silicon build only.
3. Open the `.dmg` and drag Captain to Applications.
4. If macOS says it cannot check the app, right-click Captain in Applications and choose **Open**. You can also allow it in System Settings > Privacy & Security. An unsigned build needs this once.

`Captain.app` carries Lima, the `docker` CLI, Compose, Buildx, the macOS credential helper, and the `captain` command. You do not need Homebrew.

### From source

If the releases page lists no release, or you use an Intel Mac, build the app yourself.

1. Install Rust and the Xcode Command Line Tools (`xcode-select --install`).
2. Clone the repository.
3. Run `scripts/bundle-macos.sh release`. The script downloads the tools, checks their SHA-256 sums, and builds the app.
4. Move `target/release/Captain.app` to Applications.

The tools need about 200 MB of disk. Without `release`, the script makes a debug build in `target/debug/Captain.app`.

## First launch

On the first launch, Captain shows the setup screen. It lists the CPUs, memory, and disk that Captain Engine gets. You can change them in Settings later.

1. If Captain finds another engine, such as Rancher Desktop or Docker Desktop, it lists it under **Other engines on this computer**. To copy its data after the setup, check **Bring your data along**. See [Moving from Docker Desktop or Rancher Desktop](moving-from-docker-desktop-or-rancher.md).
2. Click **Set up Captain Engine**. Captain downloads an Ubuntu image and creates the VM. The screen shows the progress.
3. Wait for the main window. The first start takes a few minutes. Later starts take seconds.

To keep your current engine instead, click **Use an existing engine**. Captain then connects to that engine, but does not start or stop it.

If the start fails, the screen shows **Captain Engine did not start** and the error. Click **Try again**. Captain keeps what the first start downloaded. If the start fails again, see [Troubleshooting](troubleshooting.md).

Captain Engine lives in `~/.captain/lima`. It never touches your own Lima or Colima VMs.

### When the engine stops

When Captain Engine is stopped, the window shows **Captain Engine is stopped**. Click **Start**. The next launch of Captain also starts the engine.

When you quit Captain, Captain Engine stops, and its containers stop with it. To keep the engine running after you quit, set `stop_engine_on_quit` to `false` in the [settings file](settings.md).

## The main window

The window has a sidebar on the left, the page on the right, and a status bar at the bottom.

### The sidebar

From top to bottom:

- **The engine header.** The engine name, its state, its CPUs, and its memory.
- **The search button.** It opens the command palette. The shortcut is ⌘K.
- **Projects.** One entry per Compose project, with "Compose · N services", how many containers run, and the published ports. **All containers**, on the same line, shows every container in one list.
- **Kubernetes namespaces.** One entry per namespace, while **Show Kubernetes containers** is on. See [Kubernetes](kubernetes.md).
- **Loose containers.** The containers that are in no project.
- **The Disk card.** How much of the engine disk is in use, and how much you can free. Click it to open [Storage](storage.md).
- **The page buttons.** Icon buttons for Images, Volumes, Networks, Snapshots, Storage, Extensions, Port Forwarding (while Kubernetes runs), Diagnostics, and Settings. Diagnostics shows a red count while a check fails.
- **The engine card.** CPU and memory gauges, and **Start**, **Stop**, or **Set up** for Captain Engine.

### The project page

Click a project in the sidebar to open its page. The page has five parts.

1. **The header.** The project folder and Compose file, and the buttons **Open folder**, **Terminal**, **Down**, and **Restart project**. While nothing runs, **Up** replaces **Restart project**.
2. **The Open row.** One pill per published port. A click opens a web port in your browser, or copies the address of a database port.
3. **The service cards.** One card per service, with its image, state, CPU, memory, and ports. A crashed service shows why it exited, for example "Exit 137 · out of memory". Each card has a logs button and a shell button. Click a card to open the inspector, with the tabs **Overview**, **Logs**, **Terminal**, **Files**, and **Stats**. Click the card again to close the inspector.
4. **The Tasks card.** Named commands from the Compose file.
5. **The project log.** One log for all services, sorted by time, with a colored tag per service.

**Terminal** opens Terminal.app in the project folder. Its `docker` commands use your default docker context, which is not always Captain Engine. To make them use Captain Engine, [set up your terminal](moving-from-docker-desktop-or-rancher.md#set-up-your-terminal).

[Projects and tasks](projects-and-tasks.md) explains the Open row, tasks, the log, and the Map tab.

### The status bar

Move the mouse over a control. The status bar shows a sentence that says what the control does, and its shortcut keys if it has any. Captain has no tooltips. The status bar does their job.

When the mouse is not over a control, the status bar shows the latest container crash, for example "worker restarted 3 times · last at 12:07:11". With no crash, it shows "Ready · hover anything for help".

The right side of the status bar shows:

- the engine and its state,
- the CPU and memory that all containers use,
- the engine disk use,
- Kubernetes on, off, starting, or failed (Captain Engine only),
- the docker context that your terminal uses.

Hover a segment to read what it means.

### The command palette

Press ⌘K. Type a page, a container name, or a command such as `restart api` or `logs worker --since 10m`. See [The command palette](command-palette.md).

## The menu bar

Captain puts an icon in the menu bar. When you close the main window, Captain and the engine keep running. To quit, press ⌘Q, or choose **Quit Captain** from the icon's menu.

### The menu

Click the icon, with either button. The menu opens. It looks like the other menus in the menu bar. Colored dots show state: green runs, amber starts, stops, or is paused, gray is stopped, and red has a problem.

- **The engine.** "Captain Engine: Running", its CPUs, and the memory the containers use. Under it, the number of running containers.
- **The problem**, only when something is wrong. A line names the worst problem, and the items under it fix it. For a container that keeps running out of memory, the menu offers **Raise Memory to** *size* (twice the old limit, and at least 512 MB), **Show Logs in a Window**, and **Stop** *name*. For a failed Diagnostics check, it offers the same fix as the Diagnostics page.
- **Start Captain Engine** or **Stop Captain Engine**, **Open Captain**, and **Settings…**.
- **Containers**, **Projects**, and **Open Ports**. Each running container has Stop, Restart, Show Logs in a Window, and its ports. Each Compose project has Start, Stop, and Restart. In Open Ports, a web port opens in your browser, and a database port copies its address.
- **Kubernetes.** Its state, a check mark that turns the cluster on or off, and **Kubernetes Contexts**.
- **Stop All Containers** and **Quit Captain**.

**Show Logs in a Window** opens a small log window that stays on top of other apps. The strip at its top shows the container's state every few seconds. Green means it runs, amber means it restarts, and red means it stopped or is unhealthy.

The Dock icon shows a count of failed checks and of containers that restart or are unhealthy.

To hide the icon, turn off **Show Captain in the menu bar** in [Settings](settings.md). Without the icon, closing the window quits Captain.

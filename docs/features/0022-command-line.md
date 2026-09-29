# Feature 0022: Command line

- Milestone: M20
- Status: Implemented; `start`, `stop`, and `restart` checked on a test VM with `--lima-home`

## Goal

A `captain` command for the terminal, like Rancher Desktop's [`rdctl`](https://docs.rancherdesktop.io/references/rdctl-command-reference): start and stop Captain Engine, show its state, change the engine settings, open a shell in the VM, and print the `DOCKER_HOST` for other tools. It works when the Captain app is closed, so scripts and CI jobs do not need the window.

## In scope

| Command | What it does |
|---------|--------------|
| `captain start` | Starts Captain Engine and prints the progress lines. It creates the VM on the first run. |
| `captain stop` | Stops Captain Engine. |
| `captain restart` | Stops, then starts Captain Engine, for example to apply new resources. |
| `captain status [--json]` | The engine choice, the Captain Engine state, the Docker socket, whether the app runs, and the Captain, Lima, and Docker versions. |
| `captain info [--json]` | The resources for the next start, this computer's CPUs and memory, the settings file, the endpoint, and on macOS `LIMA_HOME`, the instance folder, and the `limactl` in use. |
| `captain list-settings [--json]` | The settings that `set` can change, with their values. |
| `captain set <key> <value>` | Changes one setting in the settings file. |
| `captain shell [-- cmd...]` | A shell in the VM (`limactl shell captain`), or one command in it. |
| `captain docker-env` | Prints `export DOCKER_HOST='unix://…/docker.sock'`. Use it as `eval "$(captain docker-env)"`. A `'` in the path is written as `'\''`, and a path with a line break is refused. |
| `captain version` | The CLI version. `captain --version` works too. |
| `captain completion <shell>` | A completion script for bash, zsh, fish, elvish, or PowerShell. |

Settings keys for `set` and `list-settings`. Each check matches the Settings page:

| Key | Values | Check |
|-----|--------|-------|
| `engine` | `captain`, `external` | One of the two. |
| `cpus` | a whole number | 1 to this computer's CPU count (the stepper's range). |
| `memory` | GiB, for example `8` or `8GiB` | 2 GiB to three quarters of this computer's memory. |
| `disk` | GiB | 16 to 1024 GiB, and not smaller than the saved size: the disk cannot shrink. |
| `stop-engine-on-quit` | `true`, `false` | |
| `start-in-background` | `true`, `false` | `true` needs `show-menu-bar-icon`, as the page disables the switch without it. |
| `show-menu-bar-icon` | `true`, `false` | |
| `debug-logging` | `true`, `false` | |

`cpus`, `memory`, and `disk` apply on the next start. When the engine runs, `set` says to run `captain restart`.

- A new crate `crates/captain-cli` with clap's derive API. `main.rs` only parses and dispatches; each command has its own file in `src/commands/`.
- The Cargo binary is `captain-cli`, because `captain-app` already builds `target/<profile>/captain`. clap names the command `captain`, so help and completions say `captain`. `scripts/bundle-macos.sh` builds it and copies it to `Captain.app/Contents/Resources/bin/captain`, next to the bundled `docker`, and `scripts/package-macos.sh` signs it with the other tools. Users link it onto their `PATH`, as with `rdctl` ([Rancher: rdctl](https://docs.rancherdesktop.io/references/rdctl-command-reference)).
- A `--settings <path>` flag, or the `CAPTAIN_SETTINGS` variable, points the CLI at another settings file. Tests use it with a temporary folder. The app ignores both.
- Two hidden flags point the CLI at another Captain Engine instance, for tests: `--lima-home <path>` (`CAPTAIN_LIMA_HOME`) and `--instance <name>` (`CAPTAIN_INSTANCE`, default `captain`). The template, the engine lock, and the snapshots then sit next to that `LIMA_HOME`. Use a short path such as `~/.cst/lima`, because macOS limits socket paths to 104 bytes. For example: `captain --settings ~/.cst/settings.json --lima-home ~/.cst/lima --instance captain-agent-test start`.

## Out of scope

- `reset` and Kubernetes (M19). Snapshots are in feature [0023](0023-snapshots.md). `reset` deletes all containers and volumes; it stays in the app, behind a confirmation dialog, for now.
- `set` for the appearance, the accent color, the custom endpoint, and the Docker daemon settings. The daemon settings are a JSON document; a later change can add `captain set daemon <file>`.
- An HTTP API in the app, and `rdctl api` or `shutdown`.
- A `captain` command in the Linux and Windows packages. The CLI builds and its tests run there; `start` and `stop` report that the system engine is not Captain's to control, as the app does.
- `docker-env` for fish or PowerShell syntax.

## Design: the CLI works alone

`rdctl` sends most commands to the running app over a local HTTP API ([rdctl reference](https://docs.rancherdesktop.io/references/rdctl-command-reference): "The Rancher Desktop application must be running"). We looked at three options:

| Option | For | Against |
|--------|-----|---------|
| (a) The CLI works alone: reads the settings file and drives `EngineHost` itself | Works with the app closed. No server, port, or token. Reuses `captain-core` and `captain-host` as they are. | The app and the CLI can act at the same time. |
| (b) The CLI talks to the running app | One writer for everything. | Nothing works with the app closed. Needs a local API server, auth, and a protocol. |
| (c) Both | Works everywhere. | Two code paths for each command. |

We chose (a). Captain Engine is a Lima VM whose state lives in `~/.captain/lima`, not in the app, so the CLI can drive it directly, the same way the app does. Two small locks make it safe to run next to the app:

1. **An engine lock.** `LimaHost` takes an exclusive lock on `~/.captain/captain-engine.lock` for each start, stop, reset, and resource edit, and writes `starting` or `stopping` into it. A second process that tries to act gets "Captain Engine is starting in another Captain process." A status check in another process sees the held lock and reports Starting or Stopping, so the app's sidebar and menu bar follow a `captain start`. Without the lock, two `limactl start` runs on one instance race each other; Lima has no lock for this ([lima#4407](https://github.com/lima-vm/lima/issues/4407) shows `limactl list` seeing a half-made instance, and [lima#4929](https://github.com/lima-vm/lima/issues/4929) a stale disk lock).
2. **An app lock.** The app holds an exclusive lock on `app.lock` next to `settings.json` while it runs. The app keeps its settings in memory and writes the whole file on each change (ADR 0004), so a `captain set` while the app runs would be lost on the app's next save, and the app would not see it. So `set` refuses while the lock is held: "Captain is running. Change this in Settings, or quit Captain first." `status` shows whether the app runs from the same lock. A second app that finds `app.lock` held exits with "Captain is already running." It tries for one second first, because a `captain status` probe holds the lock for an instant. If the lock cannot be taken for another reason, for example a read-only folder, the app logs it and runs. The CLI then cannot check the lock either, and it treats the app as running, so it refuses to write. `captain snapshot restore` takes `app.lock` itself with the note `cli-restore`, from before the swap until the engine runs again, so the app cannot start in the middle. An app that finds that note exits with "The captain command is restoring a snapshot."
3. **A settings lock.** Every command that changes `settings.json` (`set`, `kubernetes enable` and `disable`, `snapshot restore`) waits for an exclusive lock on `settings.lock` next to it, and holds it from the read to the write. It checks `app.lock` inside that lock. The app takes the same lock to read the file at launch and around each save. So two commands never lose each other's change, and an app that starts during a command reads the saved file. Each write goes to a temporary file with the process ID and a counter in its name, then a rename, so two writers never share a temporary file. A symlinked `settings.json` stays a link: the write goes to its target.

The locks use `std::fs::File::try_lock` and, for the settings lock, `File::lock` (stable since Rust 1.89, [docs](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock)): `flock` on macOS and Linux, `LockFileEx` on Windows. The operating system drops a lock when its process ends, also after a crash, so a stale lock file does no harm. The files stay on disk.

The CLI reads the settings like the app does: the resources come from `engine_resources`, or this computer's defaults, and the Docker daemon settings (feature 0020) go to the host before a start, so a CLI start applies the same `daemon.json` as the app.

## Notes

- **Finding `limactl` from the bundle.** `Bundle::from_exe` also accepts `Contents/Resources/bin/<exe>`, so the bundled CLI finds the bundled `limactl`. Captain resolves the executable path with `canonicalize`, because a link on `PATH` (for example `/usr/local/bin/captain`) is what `std::env::current_exe` can return on macOS ([docs](https://doc.rust-lang.org/std/env/fn.current_exe.html)).
- **`shell`** runs `limactl shell captain [cmd...]` with Captain's `LIMA_HOME` and the terminal's standard input and output ([limactl shell](https://lima-vm.io/docs/reference/limactl_shell/)). On Unix it replaces the `captain` process, so the exit code and signals pass through.
- **Versions in `status`.** The Docker version comes from the engine's `/version` endpoint through `captain-docker`, only while the engine runs. The Lima version comes from `limactl --version`, as on the Diagnostics page.
- **Completions** come from [clap_complete](https://docs.rs/clap_complete/latest/clap_complete/). For zsh: `captain completion zsh > "${fpath[1]}/_captain"`.
- **Exit codes.** 0 on success, 1 on a failure (the message goes to standard error), 2 for a usage error (clap's default).
- Stopping from the CLI while the app runs is fine: the app's status poll sees the engine stop. If "Stop the engine when Captain quits" is on, quitting the app still stops an engine that `captain start` started.

## Verification

Automated (`cargo test -p captain-cli -p captain-core -p captain-host`):

- Argument parsing for each command, `--json`, `--settings`, and `shell -- cmd` with flags after `--`.
- Each settings key: good values, and the range and shrink errors.
- A restore's own hold on `app.lock` keeps the app out but lets its settings change through. A lock file that cannot be read counts as held.
- `set` writes to a temporary settings file and refuses while `app.lock` is held. It waits for the settings lock and keeps another writer's change.
- The lock: a second lock on the same file fails while the first is held, and the holder note reads back.
- `Bundle::from_exe` finds the bundle from `Contents/Resources/bin/captain`.

By hand, with the CLI built by `scripts/bundle-macos.sh`:

1. Run `Captain.app/Contents/Resources/bin/captain status`. It shows the engine state, the socket, and the versions.
2. Run `captain info`, `captain list-settings`, and `eval "$(captain docker-env)"; docker ps`.
3. Quit Captain. Run `captain stop`, then `captain start`. The progress lines print, and `docker ps` works.
4. Open Captain and run `captain restart`. The sidebar shows Stopping, then Starting, then Running.
5. With Captain open, run `captain set cpus 4`. It refuses. Quit Captain and run it again. It works, and Settings shows 4 CPUs.
6. Run `captain shell -- uname -a`. It prints the VM's kernel.
7. Run `source <(captain completion zsh)` and press Tab after `captain set `.

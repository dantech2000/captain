# Feature 0023: Snapshots

- Milestone: M16
- Status: Implemented; checked with the CLI on a test VM. The page needs a check by hand in the app.
- Parity: Rancher Desktop's [Snapshots](https://docs.rancherdesktop.io/ui/snapshots) page and [`rdctl snapshot`](https://docs.rancherdesktop.io/references/rdctl-command-reference)
- Decision: [ADR 0012](../adr/0012-snapshots.md)

## Goal

A user can save the state of Captain Engine under a name, go back to it later, and delete saved states they no longer need. A snapshot holds everything inside the VM (images, containers, volumes, the Docker daemon files) and the engine's resources.

## In scope

- A **Snapshots** page. Its sidebar entry sits above Diagnostics. The page lists each snapshot with its name, description, creation date, and disk size, newest first. The header shows the free space on the disk.
- **Create snapshot…** opens a dialog with a name (the date and time by default) and an optional description. If Captain Engine runs, the dialog says that the engine stops while Captain saves the snapshot and starts again after it.
- **Restore…** on a row asks first: "The current engine state is replaced by this snapshot. Containers, images, and volumes made since then are lost. Captain Engine restarts." A checkbox, **Save the current state as a snapshot first**, is on by default.
- **Edit…** on a row opens a dialog with the name and the description. **Save** checks the name against the other snapshots and rewrites `metadata.json`. It does not stop the engine.
- **Delete…** on a row asks first, then removes the snapshot. It does not stop the engine.
- **A space warning.** When the free space is less than the bytes the engine's disk uses now, the Create dialog shows an orange note: "Only 5.1 GB is free, and the engine's disk uses 12.4 GB. The snapshot grows as the engine changes files, so the disk may fill up." Create still works. A clone shares the disk's blocks at first, but each block the engine changes afterwards takes new space.
- While a step runs, the page shows its progress line ("Stopping Captain Engine", "Saving the snapshot", "Starting Captain Engine") and disables all snapshot buttons. The Start, Stop, Restart, and Reset buttons in the sidebar, Settings, and the menu bar are off too. A failure shows as an error toast, the same as a failed engine start.
- The CLI:

  | Command | What it does |
  |---------|--------------|
  | `captain snapshot create [NAME] [--description TEXT] [--yes]` | Saves a snapshot. `NAME` defaults to the date and time. |
  | `captain snapshot list [--json]` | Lists the snapshots: name, date, size, and description. |
  | `captain snapshot restore NAME [--yes]` | Replaces the engine with the snapshot. It refuses while the Captain app runs, because the app owns the settings file. It holds `app.lock` from before the swap until the engine runs again, so the app cannot start in between (feature [0022](0022-command-line.md)). |
  | `captain snapshot rename NAME NEW_NAME [--description TEXT]` | Renames the snapshot. `--description` also replaces the description; without it the description stays. |
  | `captain snapshot delete NAME [--yes]` | Deletes the snapshot. |

  `NAME` may also be the snapshot's ID. Before `create`, the CLI prints the space warning, if any, and asks "Save the snapshot anyway?". When the engine runs, `create` asks "Captain Engine will stop and start again. Continue? [y/N]". `restore` and `delete` always ask. `--yes` skips the question; without a terminal, the command fails unless `--yes` is given.
- Two CLI overrides for tests: `--lima-home PATH` (`CAPTAIN_LIMA_HOME`) and `--instance NAME` (`CAPTAIN_INSTANCE`). They point every command at another Lima instance. Its template, engine lock, and snapshots then sit next to that `LIMA_HOME`, not in `~/.captain`. They are hidden from `--help`.

## Out of scope

- Export and import of a snapshot to another computer (ADR 0012, options C and D).
- `rdctl snapshot unlock`. Captain's engine lock is an OS file lock, so it goes away with its process.
- Snapshots on Linux and Windows. The Snapshots page says they need Captain Engine on macOS.
- Naming the running containers in the stop warning.

## Notes

- **Layout.** `~/.captain/snapshots/<uuid>/` holds `disk`, `vz-efi`, `vz-identifier`, `lima.yaml`, `lima-version`, `user`, `user.pub`, `captain-engine.yaml`, `metadata.json`, and `complete.txt` (written last). A folder without `complete.txt` is never listed and is deleted at the next create or delete. See ADR 0012 for why each file is there.
- **`metadata.json`** has the name, description, creation time (Unix seconds), Captain version, Lima version (from the instance's `lima-version` file), the disk's allocated bytes, the engine resources (CPUs, memory, disk size), and the saved Docker daemon settings (feature [0020](0020-engine-configuration.md)) and Kubernetes settings (feature [0024](0024-kubernetes.md)), with the k3s version. These are the settings for the next start, not what the copied disk has: each start applies the saved settings, so a restore that adopts them gets the engine the user had set up. Snapshots from before these two fields keep the current daemon and Kubernetes settings on restore. One file holds it all; ADR 0012 had a separate `engine.json`.
- **Cloning.** Captain runs `/bin/cp -c`, which calls `clonefile(2)` and falls back to `copyfile(2)` when the file system cannot clone ([cp(1)](https://keith.github.io/xcode-man-pages/cp.1.html), [clonefile(2)](https://keith.github.io/xcode-man-pages/clonefile.2.html)). The workspace forbids `unsafe`, so it does not call `clonefile` through `libc`. `/bin/cp` is spelled out because a GNU `cp` on `PATH` reads `-c` differently. Lima's own docs suggest `cp -c` for raw vz disks ([Lima vz docs](https://lima-vm.io/docs/config/vmtype/vz/)).
- **Space.** When the snapshot folder and the instance are on the same volume (same device number), the clone is cheap, and Captain needs 2 GiB free. On different volumes `cp` copies the data, so Captain needs the disk's allocated size plus 2 GiB, and refuses otherwise. Free space comes from `df -Pk`, as on the Diagnostics page.
- **Restore.** Captain first checks that the snapshot has every required file (`disk` and `lima.yaml`). A damaged snapshot fails with "The snapshot is damaged: … is missing." before anything changes. Captain then clones the snapshot's files into `.restore-staging` in the instance folder. It writes `.restore-backup/journal.txt`, which lists where each backup belongs, then moves each live file into `.restore-backup` and the staged file into its place, with `rename(2)`. If a rename fails, it moves the backups back. It deletes the backup folder only after every file is in place, or after every backup went back. If a backup cannot go back, the folder stays and the error names it. A restore that finds a `.restore-backup` folder refuses, because a crash may have left the only copy of the old files there; the error names the folder, and the user moves it away. Every engine start refuses with the same message, because the live files may be missing or mixed. So the restart after a failed restore, in the app or the CLI, stops at once and shows where the old files are. The CLI reports the restore error first. An old instance whose `disk` links to `diffdisk` gets a real `disk` file, and the old `diffdisk` is deleted only after the swap put that file in place. After the swap, the app and the CLI save the snapshot's resources, daemon settings, and Kubernetes settings in `settings.json` and hand them to the host, so the next start does not edit `lima.yaml` back, rewrite `daemon.json`, or move k3s to another version.
- **Stopped VM only.** The host refuses a create or restore unless `limactl list` reports the instance `Stopped` (or, for restore, missing). The UI and CLI stop the engine first and start it again after, if it ran. `limactl snapshot` does not work with vz ([lima#3682](https://github.com/lima-vm/lima/issues/3682)).
- **App-wide steps.** The page starts each step as a detached task that holds only a weak handle to the page's model. Closing the window does not cut a step off: the restore still saves the snapshot's settings, unblocks the engine controls, and starts the engine again. A window opened meanwhile shows the page as busy. Quit during a create or restore waits until the step ends, and the step then does not start the engine again.
- **Lock.** Each create, restore, and delete holds the engine lock (`~/.captain/captain-engine.lock`, feature [0022](0022-command-line.md)) with the note `snapshot`. So the app and the CLI cannot start the engine, or run a second snapshot step, at the same time. ADR 0012 had a separate `snapshots/.lock`; one lock covers both cases.
- **Edit.** Captain writes the new metadata to `.metadata.json` in the snapshot folder, then renames it over `metadata.json`, under the engine lock. A crash leaves the old or the new file, never half of one. `SnapshotList` carries the engine disk's allocated bytes for the space warning.
- **Names.** 1 to 250 characters, no leading or trailing space, unique, as in Rancher Desktop ([manager.go](https://github.com/rancher-sandbox/rancher-desktop/blob/e4c91fe/src/go/rdctl/pkg/snapshot/manager.go)).
- **Code layout.** `captain_core::snapshot` has the metadata, the name rules, the space rule, and the `EngineSnapshots` trait. `captain-host/src/lima/snapshot` has the file plan, the copy, the swap, and `LimaSnapshots`. `EngineHost::snapshots()` returns it for `LimaHost` and `None` elsewhere. `captain-ui/src/snapshots` has the page, its model, and the dialogs. `captain-cli/src/commands/snapshot` has the four commands.

## Verification

Automated (`cargo test -p captain-core -p captain-host -p captain-cli`):

- Name rules, the default name, metadata JSON round trip, the space rule, and the space warning.
- Edit: a rename, a trimmed description, and a refused name that another snapshot has.
- The file plan for a current and a legacy (`disk` → `diffdisk`) instance, and the restore swap with its rollback, on a temporary folder.
- A snapshot without `disk` is not restorable, and a leftover `.restore-backup` stops the next restore and stays. It also stops an engine start.
- `captain snapshot restore` with a settings file that is not valid JSON fails before it looks up the snapshot.
- Restore adopts the daemon and Kubernetes settings from the metadata, and keeps the current ones for an old snapshot.
- Listing skips incomplete folders; CLI argument parsing for the four commands and the overrides.

Live: `cargo test -p captain-host --test live_snapshot -- --ignored` makes a `captain-agent-test` VM in `/tmp`, saves a snapshot, adds a volume, restores, checks that the volume is gone, and deletes the snapshot and the VM.

Live with the CLI, with a test VM (`captain-agent-test` in `LIMA_HOME=$HOME/.cst/lima`):

1. `captain --lima-home ~/.cst/lima --instance captain-agent-test start`, then `stop`, then `restart`.
2. `captain … snapshot create base --yes`. The engine stops, the snapshot saves, and the engine starts again.
3. `docker volume create captain-agent-snap` on the test VM's socket.
4. `captain … snapshot restore base --yes`. The volume is gone after the restart.
5. `captain … snapshot list`, then `snapshot delete base --yes`. The list is empty.

Rename was checked the same way on a test VM (`captain-agent-daemon` in `~/.clo/lima`): `snapshot create base -d first --yes`, `snapshot rename base clean -d renamed`, `snapshot rename clean clean2` (the description stays "renamed"), `snapshot list --json`, and `snapshot delete clean2 --yes`.

By hand in the app:

1. Open **Snapshots**. The page is empty and shows the free space.
2. Click **Create snapshot…**, type a name, and click **Create**. The engine stops, the row appears, and the engine starts again.
3. Create a volume. Click **Restore…** on the row, and confirm. A "Before restore" snapshot appears, the engine restarts, and the volume is gone.
4. Click **Edit…** on a row, change the name and the description, and click **Save**. The row shows them. A name that another snapshot has shows a red message.
5. Click **Delete…** on a row, and confirm. The row goes away.
6. With less free space than the engine's disk uses, click **Create snapshot…**. The dialog shows the orange space warning.

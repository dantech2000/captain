# ADR 0012: Snapshots as APFS clones of the stopped instance

- Status: Accepted (implemented in M16, with the changes listed at the end)
- Date: 2026-09-29

## Context

M16 adds snapshots: save the state of Captain Engine under a name, restore it later, and delete it. Rancher Desktop has the same feature. A snapshot there holds "the current configuration of your virtual machine and all associated settings", and a restore "will replace your current installation including preference settings" ([Rancher Desktop snapshots](https://docs.rancherdesktop.io/ui/snapshots)). Each snapshot has a name and an optional description, and the app is unavailable while it creates or restores one.

Captain Engine is the Lima instance `captain` in `~/.captain/lima`, with `vmType: vz` (ADR 0008). Lima 2.2 keeps these files in the instance folder ([Lima internals](https://github.com/lima-vm/lima/blob/v2.2.0/website/content/en/docs/dev/internals.md), [filenames.go](https://github.com/lima-vm/lima/blob/v2.2.0/pkg/limatype/filenames/filenames.go)):

- `disk`: the VM disk. On an instance made before Lima 2.1, `disk` can be a link to the legacy `diffdisk`. The legacy `basedisk` held the downloaded image.
- `lima.yaml`: the instance config, with CPUs, memory, and disk size.
- `vz-efi`: the EFI variable store. `vz-identifier`: the VM's machine identifier.
- `cidata.iso`, logs, PID files, and sockets, which Lima writes again on each start.

For vz the disk is a sparse raw file, because `vmOpts.vz.diskImageFormat` defaults to `raw` ([default.yaml](https://github.com/lima-vm/lima/blob/v2.2.0/templates/default.yaml)). On this machine `ls` shows 69 GB, but only the written blocks use space.

### `limactl snapshot` does not work with vz

`limactl snapshot create`, `apply`, `delete`, and `list` exist, but the vz driver returns "unimplemented" for all four ([vz_driver_darwin.go at v2.2.0](https://github.com/lima-vm/lima/blob/v2.2.0/pkg/driver/vz/vz_driver_darwin.go)). The issue for this error says "Snapshots only work with QEMU instances (`limactl create --vm-type=qemu`)" ([lima#3682](https://github.com/lima-vm/lima/issues/3682), open). QEMU snapshots are internal qcow2 snapshots inside `diffdisk` ([lima#368](https://github.com/lima-vm/lima/issues/368)), and Captain's disk is not qcow2. A proposal for the krunkit driver takes the same approach as this ADR: a `clonefile` copy of the stopped raw disk ([lima#5533](https://github.com/lima-vm/lima/issues/5533), open). Nobody has proposed one for vz.

### Options

| Option | How it works | Speed and space | Verdict |
|--------|--------------|-----------------|---------|
| A. Clone the stopped instance's files | Captain copies `disk`, `lima.yaml`, `vz-efi`, `vz-identifier`, and its own engine settings into a snapshot folder with `clonefile(2)` (what `cp -c` does). | Near instant. The clone shares blocks with the disk until one side writes. | Chosen |
| B. `limactl clone` | Lima clones the stopped instance into a second instance, for example `captain-snap-1`. | Also a copy-on-write clone: Lima's `CopyFile` "attempts copy-on-write when supported by the filesystem" ([clone.go](https://github.com/lima-vm/lima/blob/v2.2.0/pkg/instance/clone.go)). | Rejected |
| C. Export a disk image | Captain writes the disk to a compressed file, or converts it with `qemu-img`. | Minutes, and a full-size copy of the used data. Portable to another Mac. | Rejected for snapshots |
| D. Docker-level export | `docker save` for images and a tar of each volume, the same streams as the Migration Assistant (ADR 0009). | Slow. Leaves out container filesystems, `daemon.json`, the Kubernetes state, and the VM config. | Rejected for snapshots |

Option B looks close to option A, but:

- Each snapshot becomes a Lima instance in `~/.captain/lima`. `limactl list` shows it, and a user can start it by mistake. It would then run with the same forwarded socket path template and home mount as the real engine.
- Lima clears `vz-identifier` on clone (`NullifyOnClone` in filenames.go), so a restore gives the VM a new machine identity.
- There is no place for a description or for Captain's own settings.
- Instance names follow Lima's rules and the socket path limit, not the names users type.

Option A is also what Rancher Desktop does on macOS. It copies `disk`, `iso`, `lima.yaml`, `_config/override.yaml`, the SSH key pair in `_config`, and `settings.json` into a snapshot folder. It uses `clonefile` for the disk files, and falls back to a plain copy when the file system returns `ENOTSUP` or `EXDEV` ([snapshotter_unix.go](https://github.com/rancher-sandbox/rancher-desktop/blob/e4c91fe/src/go/rdctl/pkg/snapshot/snapshotter_unix.go), [copyFile_darwin.go](https://github.com/rancher-sandbox/rancher-desktop/blob/e4c91fe/src/go/rdctl/pkg/snapshot/copyFile_darwin.go)). Lima's own docs recommend `cp -c` to copy raw vz disks without copying the full virtual size.

Options C and D answer a different question: moving an engine to another computer, or keeping a backup off the disk. A snapshot does not survive a disk failure, because it shares blocks with the disk. Export can come later as its own feature.

## Decision

Captain makes snapshots by cloning the files of the stopped `captain` instance with `clonefile(2)` (option A).

### Where snapshots live

- `~/.captain/snapshots/<uuid>/`, one folder per snapshot.
- The folder is next to `~/.captain/lima` on the same APFS volume, so `clonefile` works. `clonefile` fails with `EXDEV` across volumes. If the user moves `~/.captain/snapshots` to another volume with a link, Captain falls back to a full copy and checks the space for it first (below).
- Captain names folders by UUID, so a name can hold any text and a rename changes one file.

### What a snapshot holds

| File | Source | Copy |
|------|--------|------|
| `disk` | `~/.captain/lima/captain/disk`, following the link to `diffdisk` on an old instance | `clonefile` |
| `vz-efi` | the instance folder | `clonefile` |
| `vz-identifier` | the instance folder | plain copy |
| `lima.yaml` | the instance folder | plain copy |
| `user`, `user.pub` | `~/.captain/lima/_config` | plain copy, mode `0600` for `user` |
| `captain-engine.yaml` | the template Captain wrote (`~/.captain/captain-engine.yaml`) | plain copy |
| `engine.json` | Captain's engine settings: `engine_resources` and, after M19, the Kubernetes settings | written by Captain |
| `metadata.json` | name, description, creation time, Captain version, `lima-version`, and the disk's allocated size | written by Captain |
| `complete.txt` | written last | written by Captain |

The disk holds everything inside the VM: images, containers, volumes, `daemon.json`, and the k3s cluster (ADR 0010).

A snapshot does not hold the rest of `settings.json`. Appearance, accent, and menu bar settings are the user's, not the engine's. This differs from Rancher Desktop, which restores all its preferences.

Captain leaves out `cidata.iso`, logs, PID files, and sockets. Lima writes them again on the next start.

A folder without `complete.txt` is an incomplete snapshot. Captain never offers it for restore and deletes it on the next launch. Rancher Desktop uses the same marker file.

### The VM must be stopped

A clone of a running disk is not consistent. Lima also refuses to clone a running instance. Before a create or a restore, Captain:

1. Shows a confirmation that says Captain Engine will stop, and names the containers that will stop with it.
2. Stops the engine through `EngineHost::stop`, and waits for `limactl list` to report `Stopped`.
3. Blocks Start, Stop, Reset, and resource changes in the UI and the menu bar until the snapshot step ends. Rancher Desktop has open issues for missing blocks and missing feedback here ([rancher-desktop#5998](https://github.com/rancher-sandbox/rancher-desktop/issues/5998), [#5726](https://github.com/rancher-sandbox/rancher-desktop/issues/5726)).
4. Starts the engine again when the step ends, if it ran before.

A lock file, `~/.captain/snapshots/.lock`, stops a second Captain process, or the future `captain snapshot` CLI (M20), from running a snapshot step at the same time.

### Restore

Restore replaces the current engine with the snapshot. The current disk is lost unless the user saved it.

1. The dialog offers "Save the current state as a snapshot first", checked by default. Rancher Desktop has an open request for this ([rancher-desktop#5653](https://github.com/rancher-sandbox/rancher-desktop/issues/5653)).
2. Captain stops the engine.
3. Captain clones each snapshot file into a temp folder inside the instance folder, then renames each one over the current file. A rename in one folder is atomic, so a crash leaves each file old or new, not half written.
4. Captain writes `engine_resources` from `engine.json` into `settings.json`, so Settings and `lima.yaml` agree.
5. Captain starts the engine.

If the `captain` instance does not exist (after a reset), restore creates the instance folder from the snapshot. A snapshot from a Lima version older than 2.1 has `diffdisk`. Captain restores it as `disk`, as Rancher Desktop does with its legacy names.

### Disk space

A clone is cheap at first, but each side's writes then use new blocks. In the worst case a snapshot costs its full allocated size.

- **Create:** Captain needs 2 GiB free for the step itself. If free space is less than the disk's allocated size, Captain warns that the snapshot can grow to that size as the engine writes. The warning does not block.
- **Full copy fallback:** if `clonefile` fails, Captain needs free space of at least the allocated size plus 2 GiB, and refuses the step otherwise.
- **Restore:** Captain needs 2 GiB free. The "save first" snapshot is a clone, so it costs nothing at first.
- The Snapshots page shows each snapshot's allocated size from `metadata.json`, and the free space on the volume. It does not claim the sizes add up, because clones share blocks.

### Names and descriptions

- A name is required, 1 to 250 characters, with no leading or trailing space, and unique. These are Rancher Desktop's rules ([manager.go](https://github.com/rancher-sandbox/rancher-desktop/blob/e4c91fe/src/go/rdctl/pkg/snapshot/manager.go)).
- The description is optional plain text.
- The default name is the date and time, for example `2026-09-29 14:05`.
- Rename and editing the description change `metadata.json` only, and do not stop the engine.

### Delete

Delete asks first, then removes the folder. It does not stop the engine. Free space returns only for blocks that no other clone shares.

### Where the code goes

- `EngineHost` gains `snapshot_files()`, which returns the list of files to copy, or `None` for a host without snapshots (`SystemHost`, `UnavailableHost`).
- A `captain-core` module owns the folder layout, `metadata.json`, the name rules, and the space checks, with unit tests on a temp folder.
- `captain-host` does the `clonefile` calls through `libc` on macOS. The live test snapshots and restores its own `captain-agent-test` VM, never `captain`.

## Consequences

- Create and restore take seconds, plus the engine's stop and start time.
- Snapshots use little space at first, but they grow as the engine writes. The Snapshots page cannot give an exact total.
- Captain depends on Lima's file names. A Lima release that renames `disk` or adds a required file breaks snapshots. The minimum Lima version check (ADR 0008) and the live test catch this. `metadata.json` records the Lima version of each snapshot.
- A snapshot shares blocks with the engine disk, so it does not protect against a disk failure or a deleted `~/.captain`. It is not a backup. An export feature (options C and D) can cover that later.
- Snapshots are macOS-only until Captain has a Windows or Linux engine that it controls.
- If Lima adds snapshots for vz, Captain can move behind the same `EngineHost` call without UI changes.

## Changes during implementation

M16 built this decision ([feature 0023](../features/0023-snapshots.md)) with these changes:

- **`cp -c`, not `libc`.** The workspace sets `unsafe_code = "forbid"`, so Captain runs `/bin/cp -c` instead of calling `clonefile(2)`. `cp -c` clones, and falls back to `copyfile(2)` by itself on another volume. So Captain compares the device numbers of the two folders to decide which space rule applies. It copies every file this way, including the small ones.
- **One lock.** Snapshot steps hold the engine lock, `~/.captain/captain-engine.lock`, with the note `snapshot`, instead of a separate `snapshots/.lock`. The same lock already keeps the app and the CLI from starting the engine at the same time.
- **One metadata file.** `metadata.json` holds the engine resources too; there is no `engine.json`. The snapshot also keeps the instance's `lima-version` file.
- **The trait.** `EngineHost::snapshots()` returns an `EngineSnapshots` object (list, create, restore, delete) instead of `snapshot_files()`. The Lima file plan lives in `captain-host`, and `captain-core` has the metadata, name, and space rules.
- **Restore backups.** The swap moves each live file into `.restore-backup` before the new one takes its place, and moves them back if a rename fails.
- **Incomplete snapshots** are deleted at the next create or delete, while the lock is held, not at launch.
- **CLI.** `captain snapshot` came with M16. `restore` refuses while the app runs, because the app owns `settings.json`.
- **Left for later:** rename and editing the description, the warning when free space is below the disk's allocated size, and naming the containers in the stop warning.

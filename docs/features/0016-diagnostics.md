# Feature 0016: Diagnostics and troubleshooting

- Milestone: M15
- Status: In progress
- Parity: Rancher Desktop's [Diagnostics](https://docs.rancherdesktop.io/ui/diagnostics) and [Troubleshooting](https://docs.rancherdesktop.io/ui/troubleshooting) pages

## Goal

A user can see what is wrong with Captain, the engine, or this computer, and fix it or collect logs, without a terminal.

## In scope

- A **Diagnostics** page. Its sidebar entry sits above Settings, below the resource pages. The entry shows a red count badge while a check fails.
- An **Engine** card at the top: a state dot and the engine with its state, for example "Captain Engine · Running", "Stopped", "Not set up", or "Starting…". With Captain Engine it has **Start** (stopped or failed), **Set up** (not set up), or **Stop** and **Restart** (running). They call `HostModel::start`, `stop`, and `restart`, as the Settings engine menu does. While the engine starts or stops, a spinner shows and the buttons are off. With another engine, the card shows its name and socket and no engine buttons; **Use Captain Engine** calls `HostModel::use_captain`, as the Settings engine menu does. The app icon in the rail, the sidebar's engine header, and the status bar's engine segment open this page on a click.
- A **Checks** card under it. Each check has a state (**Passed**, **Warning**, **Failed**, or **Not applicable**), a one-line detail, and a fix button where one exists:

  | Check | Passed | Warning or failed | Fix |
  |-------|--------|-------------------|-----|
  | The engine answers | A live `info` call answers within 5 s; shows the Docker and API versions. | Failed. With Captain Engine running, the detail names the sleep issue (lima#5420). A stopped or failed Captain Engine says so. | Restart Captain Engine, or Start Captain Engine |
  | Lima (macOS, Captain Engine) | `limactl --version` is 2.2.0 or newer. | Failed when missing or older. Warning when the version cannot be read. | Copy `brew install lima` or `brew upgrade lima` |
  | Docker CLI | `docker --version` answers. | Warning: Compose projects need it (ADR 0005). | Copy `brew install docker` (macOS) |
  | Docker Compose | `docker compose version` answers. | Warning. | None |
  | Free disk space | 10 GB or more free on the disk with the home folder (`df -Pk`). | Warning under 10 GB. Not applicable on Windows for now. | None |
  | Lima log size (macOS, Captain Engine) | The `*.log` files in `~/.captain/lima/captain` use 100 MB or less. | Warning above 100 MB. | Show engine files |
  | Rosetta (Apple silicon, Captain Engine) | Rosetta for Linux is installed. | Warning: x86_64 images run slowly, and the first start can hang (lima#1202). | Copy `softwareupdate --install-rosetta --agree-to-license` |

- A check that does not apply to this OS or engine shows **Not applicable** and does not count as a failure. With another engine, the Lima, Lima log, and Rosetta checks do not apply.
- The checks run at launch, when the engine connection or Captain Engine's status changes, and on **Run again**. A connection change asks only the engine again and reuses the other facts. Nothing runs on the UI thread: the facts come from a background thread, and each command has a 5-second limit.
- The fixes reuse the existing host actions (`HostModel::start` and `restart`), the file manager, and the clipboard.
- A **Troubleshooting** card on the same page:
  - **Show logs** opens Captain's log folder.
  - **Show engine files** opens `~/.captain/lima/captain`.
  - **Debug logging**: a switch saved as `debug_logging` in the settings. It raises Captain's crates to the `debug` level at once, without a restart.
- A log file. Captain writes `captain.log` to `~/Library/Logs/Captain` on macOS, where Console.app finds it. Elsewhere it uses `Captain/logs` in the local data folder. At 10 MB the file becomes `captain.log.1`, so the logs use at most 20 MB. Stderr logging stays.

## Out of scope

- Muting a check. Rancher Desktop has it; Captain has few enough checks for now.
- Deleting Lima's logs from Captain. Lima holds them open while the VM runs, so the user stops the engine and deletes them by hand.
- Reset Kubernetes and Factory Reset (M19, and the existing "Reset Captain Engine…" in Settings).
- A free space check on Windows.

## Notes

- **Code layout.** The checks are pure functions in `captain_core::diagnostics` over a `Facts` value, with unit tests. `captain-host::probe` finds the Lima version, the free space, the Lima log size, and Rosetta. `captain_docker::docker_tools` finds the docker CLI and Compose. `captain-app` joins them into the probe that the UI runs in the background. `captain-ui/src/diagnostics` has the model, the page, and the fixes.
- **The sleep issue.** The Docker socket can stop answering after the Mac sleeps while the VM keeps running ([lima#5420](https://github.com/lima-vm/lima/issues/5420), open; the same symptom in [rancher-desktop#9839](https://github.com/rancher-sandbox/rancher-desktop/issues/9839)). The check calls the engine live, because the workspace can still show "connected". A restart brings the socket back.
- **Lima 2.2.0.** The minimum moved from `captain-host` to `captain_core::diagnostics::MINIMUM_LIMA_VERSION`, so the host and the check share it. See [lima#5210](https://github.com/lima-vm/lima/issues/5210) and feature [0013](0013-captain-engine.md).
- **Log size.** Lima writes `ha.stdout.log`, `ha.stderr.log`, and serial logs to the instance folder ([Lima internals](https://lima-vm.io/docs/dev/internals/)). Nothing limits them. In [rancher-desktop#1942](https://github.com/rancher-sandbox/rancher-desktop/issues/1942), a vmnet log grew to 264 GB.
- **Rosetta.** Lima's `vz` driver shares Rosetta for Linux with the VM through `VZLinuxRosettaDirectoryShare` ([Apple docs](https://developer.apple.com/documentation/virtualization/vzlinuxrosettadirectoryshare), [Lima multi-arch docs](https://lima-vm.io/docs/config/multi-arch/)). Its files are in `/Library/Apple/usr/libexec/oah/RosettaLinux` ([Apple forums](https://developer.apple.com/forums/thread/708009)). That folder can exist while the Mac-app translator (`/Library/Apple/usr/share/rosetta`) does not, so Captain checks the Linux one. Without it, Lima can wait at "Installing rosetta..." ([lima#1202](https://github.com/lima-vm/lima/issues/1202)).
- **Free space.** `df -Pk` has a fixed format on every POSIX system ([POSIX df](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/df.html)), so Captain reads the "Available" column and adds no crate. `sysinfo` is in the lock file only through a GPUI screen capture crate that macOS does not build.
- **No new dependencies.** The size-limited log file is about 60 lines in `captain-app`. [`tracing-appender`](https://docs.rs/tracing-appender) rotates only by time, and a size-based crate is not worth a dependency for one file. The debug switch uses `tracing_subscriber::reload` ([docs](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/reload/index.html)), which Captain already has.
- `RUST_LOG` still sets the normal level. Debug logging wins over it while the switch is on.

## Verification

1. `cargo test -p captain-core diagnostics` passes. It covers each check's states and fixes.
2. Open Captain with Captain Engine running. Diagnostics lists seven checks; the engine check shows the Docker and API versions.
3. Click **Stop** in the Engine card. A spinner shows, then the card says "Stopped" with **Start**. The engine check fails with **Start Captain Engine**, and the sidebar badge shows 1. Click the fix; the engine starts and the badge goes away.
4. Choose **Other engine** in Settings. The Lima, Lima log, and Rosetta checks show **Not applicable**.
5. Click **Show logs**. Finder opens `~/Library/Logs/Captain`, which has `captain.log`.
6. Turn on **Debug logging**. `captain.log` gets `DEBUG` lines at once. After a restart of Captain the switch is still on.
7. Click **Show engine files**. Finder opens `~/.captain/lima/captain`.
8. Click the engine segment in the status bar and the rail's app icon. Each opens Diagnostics.
9. On an Apple silicon Mac without Rosetta for Linux, the Rosetta check warns, and **Copy command** copies the `softwareupdate` command.

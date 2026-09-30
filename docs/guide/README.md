# Captain user guide

Captain runs Docker containers on your Mac. It has its own engine, **Captain Engine**: a small Linux VM with Docker that Captain starts, stops, and configures. Captain can also connect to an engine you already run.

This guide is for people who use Captain. The files in [docs/features](../features) are specs for people who build it.

## Platforms

Captain is written for macOS first. Linux and Windows builds exist, but they lack parts of this guide:

- Captain Engine runs only on macOS. On Linux, Captain uses the system `dockerd`. On Windows, Captain has no engine of its own yet.
- Snapshots, Kubernetes, and the command-line tool links need Captain Engine, so they work only on macOS.
- Linux has no menu bar icon. Windows has the icon and its menu in the notification area.
- Extension windows open only on macOS.
- The Linux and Windows packages are not tested yet.

Each page says when a feature needs macOS.

## Pages

1. [Getting started](getting-started.md): install Captain, set up Captain Engine, and learn the main window and the menu bar.
2. [Moving from Docker Desktop or Rancher Desktop](moving-from-docker-desktop-or-rancher.md): copy your data, set up your terminal, and uninstall the old app.
3. [Projects and tasks](projects-and-tasks.md): the project page, `x-captain.tasks`, the Open row, and the Map tab.
4. [Storage](storage.md): see what fills the engine disk and free space.
5. [The command palette](command-palette.md): every ⌘K command.
6. [Kubernetes](kubernetes.md): a one-node cluster in Captain Engine.
7. [Snapshots](snapshots.md): save and restore the state of Captain Engine.
8. [Extensions](extensions.md): Docker Desktop extensions in Captain.
9. [Settings](settings.md): the Settings page and the settings file.
10. [Troubleshooting](troubleshooting.md): the Diagnostics checks and their fixes, and the logs.
11. [The command line](cli.md): the `captain` command.

## Reference

- [Settings reference](../reference/settings.md): every option in `settings.json`, with its default.
- [Command reference](../reference/cli.md): every `captain` command and its options.

# Captain user guide

Captain runs Docker containers on your Mac. It has its own engine, **Captain Engine**: a small Linux VM with Docker that Captain starts, stops, and configures. Captain can also connect to an engine you already run.

This guide is for people who use Captain. The files in [docs/features](../features) are specs for people who build it.

The screenshots show the demo project in [docs/demo](../demo/README.md). You can run it to follow along.

## Platforms

Captain is written for macOS first. Linux and Windows builds exist, but they lack parts of this guide:

- Captain Engine runs only on macOS. On Linux, Captain uses the system `dockerd`. On Windows, Captain has no engine of its own yet.
- Snapshots, Kubernetes, and the command-line tool links need Captain Engine, so they work only on macOS.
- Linux has no menu bar icon. Windows has the icon and its menu in the notification area.
- The app menus (File, Edit, View, Window, and Help) are macOS only. Linux and Windows get only a Quit item. Most shortcuts work there with Ctrl; see [Keyboard shortcuts](getting-started.md#keyboard-shortcuts).
- Extension windows open only on macOS.
- The Linux and Windows packages are not tested yet.

Each page says when a feature needs macOS.

## Pages

1. [Getting started](getting-started.md): install Captain, set up Captain Engine, and learn the main window (the icon rail, the sidebar, the terminal panel, and the status bar), the menus, the keyboard shortcuts, and the menu bar icon.
2. [Moving from Docker Desktop or Rancher Desktop](moving-from-docker-desktop-or-rancher.md): copy your data, set up your terminal, and uninstall the old app.
3. [Projects and tasks](projects-and-tasks.md): the project page, `x-captain.tasks`, the Open row, and the Map tab.
4. [Making new projects](new-projects.md): the New sheet (⌘N) for running an image, starting from a template, opening a folder, or pasting a `docker run` command, and the projects Captain remembers.
5. [Editing a project's files](editing-projects.md): the Files tab, checks as you type, completion, Save and apply, and Rebuild.
6. [Storage](storage.md): see what fills the engine disk and free space.
7. [The command palette](command-palette.md): every ⌘K command.
8. [Kubernetes](kubernetes.md): a one-node cluster in Captain Engine.
9. [Snapshots](snapshots.md): save and restore the state of Captain Engine.
10. [Extensions](extensions.md): Docker Desktop extensions in Captain.
11. [Settings](settings.md): the Settings page and the settings file.
12. [Troubleshooting](troubleshooting.md): the Diagnostics checks and their fixes, and the logs.
13. [The command line](cli.md): the `captain` command.
14. [AI agents](agents.md): let Claude Code, Codex, Cursor, and other agents read and run your projects through `captain mcp`.

## Reference

- [Settings reference](../reference/settings.md): every option in `settings.json`, with its default.
- [Command reference](../reference/cli.md): every `captain` command and its options.
- [MCP tool reference](../reference/mcp.md): every tool of `captain mcp` and its arguments.

# Roadmap

Captain grows one milestone at a time. Each milestone has a GitHub milestone, and each feature in it has a spec in [docs/features](docs/features) before code starts.

| # | Milestone | Scope | Status |
|---|-----------|-------|--------|
| M1 | Skeleton + container list | Workspace, CI, window with sidebar, engine status, live read-only container table | Done |
| M2 | Container actions | Start, stop, restart, pause, remove. Confirmation dialogs and error notifications. | Done: start, stop, restart, pause, resume, and delete with a confirmation dialog, project actions, and error notifications. See [0006](docs/features/0006-container-actions.md). |
| M3 | Container detail | Detail pane with Logs (streaming), Inspect (JSON), and Stats (charts) tabs | In progress: Overview, Logs, and Stats tabs work. Logs have search, timestamps, follow, and copy. See [0002](docs/features/0002-v2-interface.md) and [0007](docs/features/0007-logs.md). |
| M4 | Images | List, pull with progress, remove, prune, inspect | Done: list, filter, pull with progress, remove, prune dangling, an inspector with config and layers, and Run container. See [0003](docs/features/0003-images.md). |
| M5 | Volumes and networks | List, create, remove, prune, inspect | Done: list, filter, create, remove, prune, and a detail panel with the containers that use each volume or network. See [0004](docs/features/0004-volumes-networks.md). |
| M6 | Compose projects | Group containers by Compose project. Stack up, down, and restart. | Done: project cards with folder and services, Up, Down, Stop, Restart, and Pull through the `docker compose` CLI, a sidebar project filter, and palette commands. See [0010](docs/features/0010-compose-projects.md) and [ADR 0005](docs/adr/0005-compose-via-cli.md). |
| M7 | Settings and contexts | Theme choice, switching engines and Docker contexts, remote hosts | In progress: appearance, accent color, engine info, switching to a detected or custom endpoint, Retry. See [0008](docs/features/0008-settings.md). |
| M8 | Exec terminal | Interactive shell in a container, with the emulator behind a trait: `alacritty_terminal` now, `libghostty-vt` once Zig is in the toolchain | In progress: the Terminal tab opens bash or sh in a running container, with colors, keys, copy and paste, selection, history, resize, and Reconnect. Needs a manual check. See [0011](docs/features/0011-terminal.md) and [ADR 0007](docs/adr/0007-terminal-emulator.md). |
| M9 | Packaging | Release builds: `.dmg`, `.msi`, AppImage, `.deb` | Planned |
| M10 | Menu bar | A menu bar (tray) icon with engine status, running containers, quick actions, and Quit. See [0009](docs/features/0009-menu-bar.md). | In progress: the icon, status lines, Open Captain, Settings, container and project submenus, and Quit on macOS and Windows. Closing the window keeps Captain running. Needs a manual check; no Linux tray yet. See [ADR 0006](docs/adr/0006-menu-bar.md). |
| M11 | Icon theme | A custom Captain icon set for every glyph in the app (navigation, actions, states, empty states, the tray), drawn to match the Glass helm app icon. See [0012](docs/features/0012-icon-theme.md). | Planned |
| M12 | Captain Engine | Captain starts, stops, and configures its own engine (a Lima VM on macOS) behind an `EngineHost` interface, with a first-launch setup. See [ADR 0008](docs/adr/0008-captain-engine.md) and [0013](docs/features/0013-captain-engine.md). | In progress: the `EngineHost` trait, the Lima host (`~/.captain/lima`, Lima 2.2 or newer), the setup, starting, and stopped screens, Start and Stop in the sidebar, Settings, and the menu bar, resources, reset, and stop on quit. The live VM test passes; the app needs a manual check. |
| M13 | Migration Assistant | Copy volumes, images, networks, Compose projects, and containers from another engine into Captain Engine, never touching the source. See [ADR 0009](docs/adr/0009-migration.md) and [feature 0014](docs/features/0014-migration-assistant.md). | In progress |

## Workflow for a feature

1. Write `docs/features/NNNN-name.md`: the goal, what is in scope, what is out, and how to verify it.
2. If the feature forces an architecture decision, write an ADR in `docs/adr/`.
3. Open a GitHub issue for each step and attach it to the milestone.
4. Build, test, and update the status column above.

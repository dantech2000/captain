# Roadmap

Captain grows one milestone at a time. Each milestone has a GitHub milestone, and each feature in it has a spec in [docs/features](docs/features) before code starts.

| # | Milestone | Scope | Status |
|---|-----------|-------|--------|
| M1 | Skeleton + container list | Workspace, CI, window with sidebar, engine status, live read-only container table | Done |
| M2 | Container actions | Start, stop, restart, pause, remove. Confirmation dialogs and error notifications. | In progress: start, stop, restart, and delete (stopped only) work. See [0002](docs/features/0002-v2-interface.md). |
| M3 | Container detail | Detail pane with Logs (streaming), Inspect (JSON), and Stats (charts) tabs | In progress: Overview, Logs, and Stats tabs work. See [0002](docs/features/0002-v2-interface.md). |
| M4 | Images | List, pull with progress, remove, prune, inspect | Planned |
| M5 | Volumes and networks | List, create, remove, prune, inspect | Planned |
| M6 | Compose projects | Group containers by Compose project. Stack up, down, and restart. | Planned |
| M7 | Settings and contexts | Theme choice, switching engines and Docker contexts, remote hosts | Planned |
| M8 | Exec terminal | Interactive shell in a container, built on `libghostty-vt` | Planned |
| M9 | Packaging | Release builds: `.dmg`, `.msi`, AppImage, `.deb` | Planned |
| Later | VM management | Captain starts and manages its own engine VM on macOS and Windows | Idea |

## Workflow for a feature

1. Write `docs/features/NNNN-name.md`: the goal, what is in scope, what is out, and how to verify it.
2. If the feature forces an architecture decision, write an ADR in `docs/adr/`.
3. Open a GitHub issue for each step and attach it to the milestone.
4. Build, test, and update the status column above.

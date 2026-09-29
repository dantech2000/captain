# Feature 0010: Compose projects

- Milestone: M6 (Compose projects)
- Status: Done
- Builds on: [0006](0006-container-actions.md), which added engine-only Stop all and Restart all
- Decision: [ADR 0005](../adr/0005-compose-via-cli.md), Compose actions through the `docker compose` CLI

## Goal

Treat a Compose project as one thing. Captain shows each project's folder and services, runs Up, Down, Stop, Restart, and Pull on it, and filters the Containers page to one project.

## In scope

- Model: `ComposeProject` in `captain-core`, built from container labels. It has the name, the working directory, the config files, the services with their containers, and a status (running, partial, or stopped). `Container` gets the service, working directory, and config file labels through the Docker mapping.
- Runner: a `ProjectRunner` trait in `captain-core`, and `ComposeCli` in `captain-docker`, which runs `docker compose` against the connected endpoint.
- Project cards: the header shows the working directory (with `~` for the home folder) and the service count. The buttons are Up, Stop (while any container runs), Restart, Pull, and Down. Down asks first: "Stop and remove the containers of shop? Volumes stay."
- Progress: while a command runs, the card shows a spinner and a label such as "Pulling...". The buttons come back when it ends.
- Notifications: a failure shows an error toast with the Compose error line. It stays until the user closes it. A success shows a short toast, for example "Pulled the images of shop."
- Rows: a row in a project card shows the service name next to the container name when the two differ.
- Sidebar: a click on a project opens the Containers page with only that project. A "Project: shop" chip with a close icon next to the filter clears it.
- Command palette: "Up project shop", "Down project shop", and "Restart project shop". Down opens the same confirmation.
- No CLI: without `docker compose`, cards keep Start all or Stop all, and Restart all, through the engine, with an info icon that explains why Up, Down, and Pull are missing.

## Out of scope

- Opening or editing the Compose files.
- `up` with `--build`, `--force-recreate`, or profiles.
- Removing volumes with Down (`down -v`).
- Live Compose output in the window. Captain shows only the result.
- The legacy `docker-compose` v1 binary.

## Verification

1. In a new folder, write a `compose.yaml` with one `busybox` service that runs `sleep 300`. Run `docker compose -p captain-demo up -d`.
2. The Containers page shows a `captain-demo` card with the folder path and "1 service · 1 of 1 running".
3. Click Stop on the card. The card shows "Stopping...", then "0 of 1 running".
4. Click Up. The container runs again.
5. Click Down, then Cancel. Nothing changes. Click Down, then Down. The card goes away and a toast says "Removed the containers of captain-demo."
6. Run Up again from the folder. Click `captain-demo` in the sidebar. Only its card shows, and a "Project: captain-demo" chip shows. Click the chip. All cards come back.
7. Press ⌘K and type "restart project captain". Run the command. The container restarts.
8. Delete the `compose.yaml` and click Up. An error toast says Captain cannot find the file.
9. Run `cargo test -p captain-docker --test live_compose -- --ignored`. The test writes the `captain-agent-compose` project in a temp folder, runs Up, Stop, Restart, Pull, and Down on it, and runs Down again at the end.

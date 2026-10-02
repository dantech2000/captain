# Feature 0001: Live container list

- Milestone: M1
- Status: Done (M1). The v2 interface ([0002](0002-v2-interface.md)) and the project-first window ([0030](0030-project-window.md)) later replaced this sidebar. The engine state now shows in the status bar ([0029](0029-status-bar.md)).

## Goal

Open Captain and see every container on the engine, with the list staying current without a manual refresh.

## In scope

- Connect to the engine with the discovery order in the README.
- A sidebar with Containers, Images, Volumes, Networks, and Compose. Only Containers works in M1. The others are disabled.
- An engine status line at the bottom of the sidebar: the engine version when connected, or an error.
- A table with Name, Image, State, Status, Ports, and Created columns. It shows stopped containers too.
- Live updates from the Docker `/events` stream. A create, start, stop, die, or destroy event triggers a reload of the list.
- An empty state when there are no containers.
- An error state when the engine is not reachable.

## Out of scope

- Container actions (M2).
- A detail pane (M3).
- Reconnect when the engine restarts. For M1, restart Captain.

## Verification

1. Start an engine and run `cargo run -p captain-app`. The table shows the same containers as `docker ps -a`.
2. Run `docker run -d --name captain-test nginx`. A new row appears.
3. Run `docker stop captain-test`. The row state changes to `exited`.
4. Run `docker rm captain-test`. The row disappears.
5. Stop the engine and start Captain. The error state shows, and the app does not crash.

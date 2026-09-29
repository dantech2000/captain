# Feature 0006: container actions

- Milestone: M2 (container actions)
- Status: Done
- Builds on: [0002](0002-v2-interface.md), which added Start, Stop, Restart, and Delete for stopped containers

## Goal

Finish M2. Captain can pause and resume a container, delete a running container after a confirmation, act on a whole Compose project, and report failures as notifications.

## In scope

- Pause and Resume: `ContainerAction::Pause` and `Unpause`, in the Docker engine (`pause_container`, `unpause_container`) and the fake engine.
- Inspector actions: five equal buttons, Start or Stop, Pause or Resume, Restart, Browser, and Delete. Pause is disabled unless the container runs. Restart works only on a running container.
- Row actions: the selected row shows Start or Stop, Pause or Resume (running and paused containers only), and Restart.
- Delete confirmation: Delete opens a dialog, "Delete captain-web? This removes the container. Its volumes stay.", with Cancel and a red Delete button. For a running or paused container the red button is "Stop and delete". It sends `ContainerAction::ForceRemove`, a remove with `force: true`.
- Notifications: a failed action shows an error toast with the engine's message. The toast stays until the user closes it. A delete shows a short success toast. The workspace emits `WorkspaceEvent`, and `AppShell` turns each event into a toast.
- Project actions: a Compose card header has Restart all, and Stop all (or Start all when no container runs). Each runs the action on every container in the group. The Standalone card has no project actions.
- Collapsible cards: a click on a card header folds its rows. The chevron points right while the card is folded. The workspace keeps the folded cards, so a card stays folded when the list reloads.

## Out of scope

- Compose-level `up`, `down`, and recreate (M6).
- Removing volumes with the container.
- Undo for a delete.

## Verification

1. Run `docker run -d --name captain-web -p 8089:80 nginx:alpine` and select the new row.
2. Click Pause. The state changes to paused and the button changes to Resume. Click Resume. The state changes back to running.
3. Click Delete. The dialog offers Cancel and "Stop and delete". Click Cancel. The container stays.
4. Click Delete, then "Stop and delete". The row goes away and a "Deleted captain-web." toast shows.
5. Stop a container and click Delete. The dialog offers Delete. Confirm. The container goes away.
6. On a Compose project card, click Stop all. Every container in the project stops, and the button changes to Start all.
7. Click a card header. The rows fold and the chevron points right. Click again to unfold.
8. Make an action fail, for example with `docker rm -f` on a container while Captain restarts it. An error toast shows the engine's message.
9. Run `cargo test -p captain-docker --test live_actions -- --ignored`. The test creates `captain-agent-actions` from `busybox`, runs every action on it, and removes it.

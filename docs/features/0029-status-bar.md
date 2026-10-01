# Feature 0029: Status bar with hover help

- Milestone: M23
- Status: Implemented. It needs a check by hand in the app.
- Design: the `Manifest` screen of the v3 canvas (its `<footer aria-label="Status bar">`). See [0027](0027-v3-interface.md), design rule 5.

## Goal

Every control says what it does. A 30 px bar at the bottom of the main window shows the help sentence of the control under the mouse, and the state of the engine the rest of the time. Hover text moves out of tooltip bubbles and into the bar.

## In scope

- **The bar.** It sits under the sidebar and the page, across the full width of the main window. It uses the sidebar color, with a line on top.
- **Left side, while the mouse is over a control:** an info icon, the control's help sentence, and its shortcut keys as small key chips when it has any (for example ⌘K on the search button, ⌫ on the parent-folder button).
- **Left side, otherwise:** the latest notable container event, for example "worker restarted 3 times · last at 12:07:11", with a red dot. With no such event, "Ready". Both end with "· hover anything for help".
  - A container that exits on its own is notable. A `docker stop`, `docker kill`, or `docker restart` is not: Docker sends `kill` before `die` for those, and only `die` for a crash or an out-of-memory kill ([docker events](https://docs.docker.com/reference/cli/docker/system/events/)).
  - A `start` after such an exit counts as a restart. The time is when Captain received the event, because `EngineEvent` has no time.
- **Right side:** segments, each with its own help sentence.
  - The engine: a colored dot and "Captain Engine", or the name of the external engine from its socket (Docker Desktop, OrbStack, Colima, Rancher Desktop, Podman, Docker Engine, or Remote engine).
  - CPU and memory use of all containers, against the engine's CPUs and memory. They show only while connected.
  - The engine disk use, from feature 0031.
  - Kubernetes on, off, starting, or failed. It shows only with Captain Engine.
  - The docker CLI's current context. Captain reads it again when the engine endpoint changes.
- **The hover API** (`crate::help`):
  - `HoverHelp` is an app-wide entity with the current hint and the element that set it.
  - `HelpExt` adds `.help(text)` and `.help_keys(text, &[CMD, "K"])` to any element with an id. It uses GPUI's `StatefulInteractiveElement::on_hover` (gpui-pre 0.3.7, `src/elements/div.rs`).
  - `HoverHelp` keeps the hints of all hovered elements, innermost last, and shows the last one. A hover-out removes only the leaving element's hint, so moving between two controls does not flicker, and leaving a control inside another (a switch in a Settings row) shows the outer hint again.
  - Only the status bar observes `HoverHelp`, so a new hint redraws the bar and nothing else.
  - A mouse down anywhere in the window clears the hint, because a click can remove the control under the mouse, and a removed control never reports its hover-out.
  - An element that disappears without a click (a list refresh, a cleanup) never reports its hover-out either. Each hint's hover listener holds a token (`help/liveness.rs`); GPUI drops the listener after the first frame without the element. After each frame with a hint, `hover_batch` drops the hints whose element has no live token.
- **Help sentences** on: the sidebar pages, projects, search button, and the parts of the status line; container row buttons and ports; Compose card buttons; the inspector buttons and tabs; the log, file, and image inspector tools; the toolbars of Images, Volumes, Networks, Snapshots, Extensions, Port Forwarding, and Diagnostics; the engine start and set-up screens.
- `icon_button`, `action_button`, and `primary_button` take a help argument, so each of their buttons has a sentence.
- **More help sentences** (the polish pass):
  - Each choice of a segmented control: the container, image, volume, and network filters, the scan severities, Appearance, the engine choice, the Run dialog's restart policy, and the Migration Assistant's image choice. `Segment` has a `help` field.
  - The "Show Kubernetes containers" checkbox and the restore dialog's "Save the current state first" checkbox, through a wrapper `div` with an id, because gpui-kit's `Checkbox` has no hover listener.
  - Every Settings row with a control, on the whole row, and each theme card.
  - Cancel and the confirm button of the alert dialogs that remove or reset something: delete container, delete selected containers, Down, delete and prune volumes, delete snapshot, remove extension, reset Captain Engine and Kubernetes, the socket link, and the migration switch-over. `danger_footer` builds their footer from gpui-kit's `DialogClose` and `DialogAction`, because the default footer's buttons take no help.
  - The Create snapshot, Restore snapshot, and Storage review dialogs' own buttons.
- All `Tooltip` uses are gone.

## Out of scope

- Help in dialogs that only collect input (Run, Build, Tag, Push, Forward, Install extension). Their buttons keep gpui-kit's defaults.
- The same sentences in the ⌘K palette (M27).
- The extension windows. They have no status bar.

## Notes

- GPUI allows one hover listener per element, so call `.help` once per element. Do not chain it onto a wrapper that already takes a help argument.
- GPUI keeps hover state per element id. Two visible elements with the same id share it.
- A help sentence with live data (counts, names, bytes that can be freed) is fixed when the mouse enters. It stays stale until the next hover. This is a known limitation.
- A layer's full command in the image inspector now shows in the bar, where a long command is cut at the window's width.

## Verification

1. Run `cargo test -p captain-ui help::` and `cargo test -p captain-ui latest_event`.
2. Open Captain. Check that the bar shows "Ready · hover anything for help" and the engine, CPU, memory, Kubernetes, and context segments.
3. Move the mouse over each sidebar entry, then down the list without a stop. Check that the sentence changes with no blank frame in between.
4. Hover the search button. Check that the bar shows the ⌘ and K key chips.
5. Hover a Compose card's Down button. Check that the sentence names the project and its container count.
6. Run `docker run -d --name captain-agent-crash --restart on-failure:3 alpine sh -c 'sleep 2; exit 1'`. Wait ten seconds. Check that the bar shows "captain-agent-crash exited again at …, after 3 restarts". Remove the container.
7. Stop a container with its Stop button. Check that the bar does not show it as an event.
8. Check that no tooltip bubble appears anywhere in the main window.
9. Hover each segment of the Containers filter, then "Show Kubernetes containers". Check the sentences.
10. In Settings, hover a row, then its switch or button, then the row again. Check that the row's sentence comes back.
11. Open the Delete dialog of a container. Hover Cancel and Delete. Check the sentences. Click Delete; check that the container goes. Open it again and press Escape; check that nothing changes.

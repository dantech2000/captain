# Feature 0002: v2 interface

- Milestones: M2 (actions) and M3 (detail), built together as one pass
- Status: Done. The v3 interface ([0027](0027-v3-interface.md)) later replaced the sidebar: the pages moved to an icon rail, the projects list stayed, and the engine card went away. The engine state shows in the status bar ([0029](0029-status-bar.md)), and Start, Stop, and Restart are on the Diagnostics page ([0016](0016-diagnostics.md)).
- Design: the "v2" row of the Captain UI canvas

## Goal

Replace the M1 table with the v2 design: a branded sidebar, live stat tiles, containers grouped into project cards, and an inspector with tabs.

## In scope

- Captain theme tokens for dark and light, following the system appearance.
- Sidebar: Captain brand and engine state, navigation with counts, Compose projects, and an engine card with CPU count and memory.
- Header: title, running and stopped counts, and an All / Running / Stopped filter.
- Stat tiles: running count, total CPU, total memory, and total network rate, each with a sparkline.
- Project cards: containers grouped by the `com.docker.compose.project` label. Containers in no project go in their own "Loose containers" card.
- Rows: status dot, name, health badge, image and tag, published ports as links, CPU sparkline, memory, and uptime.
- Selection: a click on a row selects it and fills the inspector.
- Inspector header and actions: Start or Stop, Restart, Open in browser, and Delete. Delete works only on stopped containers, so it needs no confirmation dialog yet.
- Inspector tabs:
  - Overview: stat tiles, ports, health-check history (up to the last five checks the engine keeps), environment variables with secrets masked, and mounts.
  - Logs: the last 500 lines, then live lines. The level filter is All / Info / Warn / Error.
  - Stats: larger CPU, memory, and network charts for the last 60 samples. Memory leaves out the page cache the way `docker stats` does (`total_inactive_file` on cgroup v1, `inactive_file` on v2). The network rate divides each byte delta by the time between the two samples, so a gap while stats reconnect shows the average rate.
  - Terminal and Files: a placeholder that names the milestone that adds them.
- Live stats: one stats stream per running container, kept in a 60-sample history.

## Out of scope

- The ⌘K command palette (its own feature).
- Images, Volumes, and Networks pages.
- Project-level stop and restart (built later, in [0006](0006-container-actions.md)).
- The Captain Engine VM.

## Verification

1. The sidebar, tiles, cards, and inspector match the v2 artboard in dark and light mode.
2. CPU and memory values move once a second for running containers.
3. `docker run -d --name captain-test -p 8089:80 nginx:alpine` adds a row with a `:8089` port link.
4. Stop, Start, and Restart in the inspector change the row state.
5. Delete removes a stopped container, and the button is disabled for a running one.
6. The Logs tab shows new lines from `curl localhost:8089` within a second.

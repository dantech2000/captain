# Feature 0032: Menu bar popover (replaced), floating log, and Dock badge

- Milestone: M26
- Status: The popover is replaced by the native menu of [0009](0009-menu-bar.md) (the `traymenu` change). The floating log, the Dock badge, the problem logic, and the engine API stay. They need a check by hand in the app, and on Windows.
- Design: the `Lighthouse` screen of the v3 canvas. See [0027](0027-v3-interface.md) and [ADR 0006](../adr/0006-menu-bar.md).

## Goal

The daily questions are "what is running" and "which port". The menu bar icon answers them without the main window. When something breaks, it names the cause and offers the fix. A small log window can stay on top while you work, and the Dock icon counts the problems.

## Replaced: the popover

From M26, a left click on the icon opened a GPUI `WindowKind::PopUp` window under it, with the engine and a switch, a warning card, project and Kubernetes switches, open ports, a footer, and a help line. The user asked for the standard OS menu instead, like Rancher Desktop's. So a left click opens the native `muda` menu again, and every popover action moved into it:

| Popover | Native menu |
|---------|-------------|
| Engine header and switch | "Captain Engine: Running · 5 CPUs · 60 MB of 5.8 GB", and Start or Stop Captain Engine |
| Warning card and its buttons | The problem line, then Raise Memory to *size*, Show Logs in a Window, Stop or Restart *name*, or the Diagnostics fix |
| Project switches | Projects ▸, each with Start, Stop, and Restart and a status dot |
| Kubernetes switch | The Kubernetes check item and its status line |
| Open ports | Open Ports ▸, which opens web ports and copies database addresses |
| Open Captain, Float logs, Stop all | Open Captain, Show Logs in a Window per container, Stop All Containers |

The popover window, its placement, the screen list, and the icon-click handler are gone. The menu is described in [0009](0009-menu-bar.md).

## In scope

- **The worst problem.** Pure logic in `captain_core::problems::first_problem`, in this order:
  1. Captain Engine did not start: the reason, and the fix from Diagnostics (Start or Restart Captain Engine, Show Engine Files, or Copy Command).
  2. A container keeps restarting and `inspect` says the kernel killed it for memory (`State.OOMKilled`, or the `oom` event): "worker keeps restarting: out of memory at 256 MB, restarted 3 times." The fixes: raise the limit to twice the old one, at least 512 MB (`captain_core::project_map::raised_memory`, as on the project page), through `docker update`, show the logs, and stop it. With no memory limit, the engine ran out, and Raise is not offered.
  3. A container keeps restarting for another reason: show the logs, and stop it.
  4. A container fails its health check: show the logs, and restart it.
  5. Another failed diagnostics check: its title, its detail, and its fix.
- **Floating log window.** A `WindowKind::Floating` window (an `NSPanel` at the floating level on macOS), 440 by 250 px, titled with the container and its project. A health strip samples the container every 3 seconds: green runs, amber restarts or starts its health check, red stopped or unhealthy. Below it, the newest lines of the container's log as they arrive (`ContainerApi::logs_with`, the last 40 lines first), warnings and errors in their colors. When the container stops, a line says so; when it starts again, the window opens a new stream from the newest line it showed. One window per container: opening it again brings it forward. The menu, the details panel header, and the command palette open it.
- **Dock badge (macOS).** The number of failed diagnostics checks plus containers that are restarting, crashed within the last minute, or unhealthy, on the Dock icon through `NSApplication.dockTile.badgeLabel`. No badge at 0. A `docker stop` or `docker kill` clears the container's crash. The app redraws when a crash turns a minute old, so the badge, the tray, and the sidebar drop it even when no container runs.
- **Engine API.** `ContainerApi::update_memory(&self, id, memory_bytes)` sets the memory limit and a swap limit of twice the memory, like `docker run` does by default; the engine refuses a memory limit above the old swap limit. `ContainerDetail` has `oom_killed`, `memory_limit`, and `restart_count`.

## Out of scope

- A Linux tray.
- A Dock badge on Windows (a taskbar overlay icon).
- "3 times in 2 minutes": the problem line shows the engine's restart count, not a rate.
- Changing the memory of a Compose service in its Compose file. Raise changes the running container; `docker compose up` with the old file sets the old limit again.
- New settings. "Show the menu bar icon" still turns the icon off.

## Notes

- The badge count is `captain_core::problems::problem_count`; the menu takes its problem from `first_problem` with the crash tracker, so the line stays during a crash loop.
- The menu bar code inspects a container once per new crash (`crates/captain-app/src/tray/exit_facts.rs`), so a raised limit shows after the next restart.
- No `unsafe` code: `NSApplication.dockTile` and `setBadgeLabel` are safe in objc2-app-kit 0.3.2.
- Sources:
  - `NSDockTile.badgeLabel`: <https://developer.apple.com/documentation/appkit/nsdocktile/badgelabel>
  - GPUI window kinds: gpui-pre 0.3.7 `src/platform.rs` (`WindowKind`) and gpui-pre-macos `src/window.rs` (the `NSPanel` levels).
  - `docker update`: <https://docs.docker.com/reference/cli/docker/container/update/>; the memory and swap rule: <https://github.com/moby/moby/pull/25461>
  - OOM kills in events and inspect: <https://docs.docker.com/reference/cli/docker/system/events/>, <https://docs.docker.com/reference/api/engine/version/v1.47/>

## Verification

1. Run `cargo test -p captain-core problems` and `cargo test -p captain-app tray`.
2. Run `docker run -d --name captain-agent-oom --memory 64m --restart always alpine sh -c 'tail /dev/zero'`. Open the menu bar menu. Check the line "captain-agent-oom keeps restarting: out of memory at 64 MB, …" and the Dock badge.
3. Choose Show Logs in a Window. Check that a small window stays on top of other apps, with the health strip and the log. Choose it again; check that the same window comes forward.
4. Choose Stop captain-agent-oom. Check that the problem line and the badge go away. Remove the container.

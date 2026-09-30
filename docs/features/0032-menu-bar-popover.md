# Feature 0032: Menu bar popover, floating log, and Dock badge

- Milestone: M26
- Status: Implemented. It needs a check by hand in the app, and on Windows.
- Design: the `Lighthouse` screen of the v3 canvas. See [0027](0027-v3-interface.md) and [ADR 0006](../adr/0006-menu-bar.md).

## Goal

The daily questions are "what is running" and "which port". A left click on the menu bar icon answers them in a popover, without the main window. When something breaks, the popover names the cause and offers the fix. A small log window can stay on top while you work, and the Dock icon counts the problems.

## In scope

- **Left click, right click.** A left click on the icon opens the popover; a second click closes it. A right click still opens the native menu with Quit. Under its status line, the menu names the worst container problem in the popover's words: `Problem::sentence` in `captain_core::problems` is the one source for both. The menu has no diagnostics checks and no exit facts, so an out-of-memory restart reads as a plain restart there. The icon is built with `with_menu_on_left_click(false)`, and a left-button release (`TrayIconEvent::Click` with `MouseButtonState::Up`) toggles the popover. Windows works the same way. Linux has no tray (ADR 0006), so nothing changes there.
- **The popover window.** A `WindowKind::PopUp` window, 380 px wide, under the icon. On macOS GPUI makes it a non-activating `NSPanel` at the pop-up level, so it opens over full-screen apps without bringing the main window forward. Its height fits the content, up to the screen's height; the middle part scrolls. It closes when it loses focus, on Escape, or on a second click on the icon.
- **Placement.** tray-icon reports the icon's rectangle in physical pixels, in global coordinates with the origin at the top left of the primary screen. Captain divides by the screen's scale, finds the screen that holds the icon, and puts the popover 4 px under the icon, centered on it and kept 8 px inside the screen's visible frame. If there is no room below (a taskbar at the bottom), it goes above the icon. On macOS GPUI places a window relative to its display, so the rectangle is made relative; on Windows it stays global.
- **Engine header.** The engine glyph, the engine name, "Running · 5 CPUs · 463 MB of 5.8 GB" (the containers' memory use of the engine's memory), and a switch that starts or stops Captain Engine. Another engine shows the switch off and disabled.
- **Warning card.** Only when there is a problem, and only the worst one:
  1. Captain Engine did not start: the reason, and the fix from Diagnostics (Start or Restart Captain Engine, Show engine files, or Copy command).
  2. A container keeps restarting and `inspect` says the kernel killed it for memory (`State.OOMKilled`): "worker keeps restarting: out of memory at 256 MB, restarted 3 times." Buttons: **Raise to 512 MB** (twice the limit, through `docker update`), **Show logs** (the floating log), and **Stop worker**. With no memory limit, the engine ran out, and Raise is not offered.
  3. A container keeps restarting for another reason: Show logs and Stop.
  4. A container fails its health check: Show logs and Restart.
  5. Another failed diagnostics check: its title, its detail, and its fix.
- **Projects.** One row per Compose project with "4 of 5 running" or "Stopped" and a switch: on runs `docker compose up`, off runs `docker compose stop`. Without the Compose CLI, the switch starts or stops the containers through the engine. A Kubernetes row shows while Kubernetes is on in the settings or still running; its switch turns it on or off and applies at once.
- **Open ports.** Each published port of a running container, with its Compose service or container name. A click opens `http://localhost:PORT`; database and cache ports (5432, 3306, 6379, 27017, and others) copy `localhost:PORT` instead.
- **Footer.** Open Captain, Float *name* logs (for the container in the warning card, else the selected running container), and Stop all.
- **Help line.** The popover has no status bar. Every control has a `.help()` sentence, and a line at the bottom shows the sentence of the control under the mouse, from the same `HoverHelp` model.
- **Floating log window.** A `WindowKind::Floating` window (an `NSPanel` at the floating level on macOS), 440 by 250 px, titled with the container and its project. A health strip samples the container every 3 seconds: green runs, amber restarts or starts its health check, red stopped or unhealthy. Below it, the newest lines of the container's log as they arrive (`ContainerApi::logs`, the last 40 lines first), warnings and errors in their colors. One window per container: opening it again brings it forward. The close button closes it.
- **Dock badge (macOS).** The number of failed diagnostics checks plus containers that are restarting or unhealthy, on the Dock icon through `NSApplication.dockTile.badgeLabel`. No badge at 0.
- **Engine API.** `ContainerApi::update_memory(&self, id, memory_bytes)` sets the memory limit and a swap limit of twice the memory, like `docker run` does by default; the engine refuses a memory limit above the old swap limit. `ContainerDetail` gains `oom_killed`, `memory_limit`, and `restart_count`.

## Out of scope

- A Linux tray, and so a Linux popover.
- A Dock badge on Windows (a taskbar overlay icon).
- "3 times in 2 minutes": the card shows the engine's restart count, not a rate.
- Changing the memory of a Compose service in its Compose file. Raise changes the running container; `docker compose up` with the old file sets the old limit again.
- New settings. "Show the menu bar icon" still turns the icon, and so the popover, off.

## Notes

- Selection of the warning is pure logic in `captain_core::problems::first_problem`, with the badge count in `problem_count`. The placement math is in `captain-app/src/tray/placement.rs`.
- The popover inspects a container once each time it enters the restarting state, so a raised limit shows after the next restart.
- A click on the icon while the popover is open first takes focus from the popover, which closes it; the click that follows within 500 ms does not open it again.
- Two screens with different scales can both seem to hold the icon's physical rectangle. On macOS, Captain tries the screen under the mouse first (`NSEvent.mouseLocation`). GPUI does not report a display's scale on Windows, so Windows uses the scale of an open Captain window, or 1.
- No `unsafe` code: `NSScreen`, `NSEvent.mouseLocation`, `NSApplication.dockTile`, and `setBadgeLabel` are safe in objc2-app-kit 0.3.2.
- Sources:
  - tray-icon 0.25 click events and left-click menus: <https://docs.rs/tray-icon/0.25.1/tray_icon/enum.TrayIconEvent.html>, <https://docs.rs/tray-icon/0.25.1/tray_icon/struct.TrayIconBuilder.html#method.with_menu_on_left_click>
  - GPUI window kinds: gpui-pre 0.3.7 `src/platform.rs` (`WindowKind`) and gpui-pre-macos `src/window.rs` (the `NSPanel` levels).
  - `NSDockTile.badgeLabel`: <https://developer.apple.com/documentation/appkit/nsdocktile/badgelabel>
  - `NSScreen.visibleFrame` and `backingScaleFactor`: <https://developer.apple.com/documentation/appkit/nsscreen>
  - `docker update`: <https://docs.docker.com/reference/cli/docker/container/update/>; the memory and swap rule: <https://github.com/moby/moby/pull/25461>
  - OOM kills in events and inspect: <https://docs.docker.com/reference/cli/docker/system/events/>, <https://docs.docker.com/reference/api/engine/version/v1.47/>

## Verification

1. Run `cargo test -p captain-core problems` and `cargo test -p captain-app placement`.
2. Open Captain. Left-click the menu bar icon. Check that the popover opens under the icon, inside the screen, with the engine, projects, and ports. Right-click the icon; check that the menu with Quit opens.
3. Click the icon again, click another app, and press Escape with the popover open. Check that each closes the popover.
4. Hover the controls in the popover. Check that the line at the bottom shows each sentence.
5. Run `docker run -d --name captain-agent-oom --memory 64m --restart always alpine sh -c 'tail /dev/zero'`. Open the popover. Check the card "captain-agent-oom keeps restarting: out of memory at 64 MB" and the Dock badge. Click Raise to 512 MB; check `docker inspect -f '{{.HostConfig.Memory}}' captain-agent-oom` shows 536870912.
6. Click Show logs. Check that a small window stays on top of other apps, with the health strip and the log. Click Show logs again; check that the same window comes forward.
7. Click Stop captain-agent-oom. Check that the card and the badge go away. Remove the container.
8. Turn a project off and on with its switch. Click a web port and a database port.
9. Turn off "Show the menu bar icon" with the popover open. Check that the popover closes.

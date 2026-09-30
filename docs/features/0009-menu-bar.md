# Feature 0009: Menu bar

- Milestone: M10
- Status: Implemented on macOS and Windows; needs a manual check in the running app. The native menu replaced the M26 popover in the `traymenu` change.

## Goal

Reach Captain from the menu bar (macOS) or the notification area (Windows) without opening the main window, the way Rancher Desktop and Docker Desktop do.

## In scope

- A Captain icon that shows the engine state: running, starting, stopped, or needs attention.
- A left click and a right click both open the same native menu (`muda`), as macOS menu extras and Rancher Desktop do. From M26 to the `traymenu` change, a left click opened a GPUI popover; its actions now live in this menu. See [0032](0032-menu-bar-popover.md).
- The menu follows the macOS menu style: Title Case items, "…" on items that open a window or ask for more, disabled lines for status, separators between groups, and submenus for lists. Top to bottom:
  1. **Status**, not clickable: "Captain Engine: Running · 5 CPUs · 60 MB of 5.8 GB", then "3 of 5 containers running" while the engine runs.
  2. **The problem**, only when there is one: a disabled line with the worst problem in the words of `Problem::line` (`captain_core::problems`), then its fixes. Out of memory at a limit: **Raise Memory to 512 MB** (twice the limit, through `ContainerApi::update_memory`), **Show Logs in a Window**, and **Stop** *name*. A restart loop: Show Logs and Stop. A failed health check: Show Logs and **Restart** *name*. A failed engine, or a failed Diagnostics check with a fix: the Diagnostics fix, such as **Restart Captain Engine**.
  3. **Start Captain Engine** or **Stop Captain Engine** (or **Set Up Captain Engine…**), **Open Captain**, and **Settings…**.
  4. While the engine runs: **Containers ▸** (per running container: Stop, Restart, Show Logs in a Window, and its ports), **Projects ▸** (per Compose project: Start, Stop, Restart), and **Open Ports ▸** ("web — localhost:8080" opens the browser; a database port reads "db — Copy localhost:5432" and copies the address, by the Project page's rule).
  5. **Kubernetes**, with Captain Engine: a status line while it is on, a check item that turns it on or off (the Settings switch's `turn_on`, then Apply), and **Kubernetes Contexts ▸**.
  6. **Stop All Containers**, while the engine runs. It is disabled when nothing runs.
  7. **Quit Captain** (⌘Q).
- Status lights: a colored dot on the engine line (green running, amber starting or stopping, gray stopped, red failed), the problem line (red; amber for a failed health check), each container (green running, amber paused or restarting, gray stopped, red crashing or unhealthy), each project (red when a container fails, green when all run, amber when some run, gray when none), and the Kubernetes line.
- Start in a project starts only the stopped containers, and Stop stops only the running ones. An item with nothing to act on is disabled.
- Closing the main window keeps Captain running in the menu bar. Quit exits. A click on the Dock icon opens the window again.

## Out of scope

- A Linux status area icon. `tray-icon` needs a GTK main loop, which GPUI does not run. On Linux, closing the last window still quits. See ADR 0006.
- A status dot on the Kubernetes check item. muda's `CheckMenuItem` takes no image, so the dot sits on its own status line.
- Icons on Linux (GTK 4 ignores submenu icons), since Linux has no tray.

## Notes

- The design is in [ADR 0006](../adr/0006-menu-bar.md): `tray-icon` and `muda` on GPUI's main-thread run loop, menu clicks forwarded into a GPUI task, and a snapshot of the workspace that decides when to rebuild the menu.
- The code is in `crates/captain-app/src/tray/`. `menu_model.rs` builds the menu as plain data, with one file per section (`problem_menu.rs`, `containers_menu.rs`, `projects_menu.rs`, `kubernetes_menu.rs`); `menu.rs` turns it into `muda` items. The snapshot (`snapshot.rs`), gathered in `gather.rs`, holds only what the menu shows, so stats samples do not rebuild it. Only the engine line follows the stats: when nothing else changed, its text changes in place with `IconMenuItem::set_text`.
- The problem uses the crash tracker (`Workspace::recent_crash`), so it stays steady during a crash loop. `exit_facts.rs` inspects each crashing container once per new crash for `oom_killed`, `memory_limit`, and `restart_count`, which out of memory and Raise need.
- The dots are drawn in code (`dot.rs`) like the icon: a 36 px canvas with a 14 px anti-aliased dot, one cached image per color. muda scales an item image to 18 points high on macOS (so the dot is 7 points) and draws it into 16 px on Windows. They are color images, not templates. They use Apple's system colors (green #34C759, amber #FF9F0A, red #FF3B30, gray #8E8E93), not Captain's theme: the system draws the menu in its own light or dark appearance, which can differ from Captain's theme.
- The icon is a ship's wheel, drawn in code as a template image on macOS. It shows the engine state: dimmed when stopped, turning while starting, full when running, and full with a notch dot when a container needs attention. See [0012](0012-icon-theme.md).
- The app owns the `Workspace` entity, and `AppShell::with_workspace` builds a window around it. So the tray keeps working with no window open.
- Sources:
  - tray-icon opens the menu on a left click by default: <https://docs.rs/tray-icon/0.25.1/tray_icon/struct.TrayIconBuilder.html#method.with_menu_on_left_click>
  - muda item images: <https://docs.rs/muda/0.20.0/muda/struct.IconMenuItem.html>, <https://docs.rs/muda/0.20.0/muda/struct.Submenu.html#method.set_icon> (unsupported on GTK 4), and <https://docs.rs/muda/0.20.0/muda/struct.CheckMenuItem.html> (no image). The sizes: <https://docs.rs/crate/muda/0.20.0/source/src/platform_impl/macos/mod.rs> (`to_nsimage(Some(18.))`) and <https://docs.rs/crate/muda/0.20.0/source/src/platform_impl/windows/icon.rs> (a 16 by 16 bitmap).
  - Menu style (Title Case, the ellipsis): <https://developer.apple.com/design/human-interface-guidelines/menus>
  - System colors: <https://developer.apple.com/design/human-interface-guidelines/color#Specifications>

## Verification

1. Run `cargo test -p captain-app tray`.
2. Left-click the icon, then right-click it. Check that both open the same native menu, in the system's light or dark style, with colored dots.
3. Check the engine line's CPUs and memory against the status bar. Check that the container count matches the Containers page.
4. Stop and Restart a container from Containers ▸. Check the main window. Open Show Logs in a Window.
5. Open a web port and a database port from Open Ports ▸. Check that the browser opens for the first and the address is on the clipboard for the second.
6. Run `docker run -d --name captain-agent-oom --memory 64m --restart always alpine sh -c 'tail /dev/zero'`. Check the red problem line and Raise Memory to 512 MB. Choose Raise; check `docker inspect -f '{{.HostConfig.Memory}}' captain-agent-oom` shows 536870912. Choose Stop captain-agent-oom, then remove the container.
7. Turn Kubernetes on and off with its check item. Check the Settings switch follows.
8. Closing the window keeps the icon, and Open Captain brings the window back. Quit exits the app.

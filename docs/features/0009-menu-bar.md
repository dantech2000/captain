# Feature 0009: Menu bar

- Milestone: M10
- Status: Implemented on macOS and Windows; needs a manual check in the running app

## Goal

Reach Captain from the menu bar (macOS) or the notification area (Windows) without opening the main window, the way Rancher Desktop and Docker Desktop do.

## In scope

- A Captain icon that shows the engine state: running, starting, or stopped.
- Status lines at the top of the menu, not clickable:
  - "Captain Engine is running" (or starting, or stopped)
  - "3 of 5 containers running"
- Actions:
  - Open Captain (shows and focuses the main window)
  - Settings... (opens the main window on the Settings page)
- A "Containers" submenu with one entry per running container. Each entry has Stop, Restart, and "Open localhost:PORT in browser" for each published port.
- A "Projects" submenu with Start all, Stop all, and Restart all per Compose project. Start all starts only the stopped containers, and Stop all stops only the running ones. An item with nothing to act on is disabled.
- The Containers and Projects submenus show only while the engine runs.
- Quit Captain (⌘Q).
- Closing the main window keeps Captain running in the menu bar. Quit exits. A click on the Dock icon opens the window again.

## Out of scope

- Kubernetes contexts (Captain has no Kubernetes support).
- A Linux status area icon. `tray-icon` needs a GTK main loop, which GPUI does not run. On Linux, closing the last window still quits. See ADR 0006.
- Starting and stopping the Captain Engine VM. This comes with the VM milestone.

## Notes

- The design is in [ADR 0006](../adr/0006-menu-bar.md): `tray-icon` and `muda` on GPUI's main-thread run loop, menu clicks forwarded into a GPUI task, and a snapshot of the workspace that decides when to rebuild the menu.
- The code is in `crates/captain-app/src/tray/`. The menu model (`menu_model.rs`), the snapshot (`snapshot.rs`), and the icon drawing (`icon.rs`) have unit tests.
- The app owns the `Workspace` entity now, and `AppShell::with_workspace` builds a window around it. So the tray keeps working with no window open, and a new window keeps the connection.
- The icon is a ship's wheel, drawn in code as a template image on macOS. It shows the engine state: dimmed when stopped, turning while starting, full when running, and full with a notch dot when it needs attention. See [0012](0012-icon-theme.md).

## Verification

1. The icon appears in the menu bar and changes when the engine stops.
2. The container counts match the Containers page.
3. Stop and Restart in the submenu change the container state in the main window.
4. Closing the window keeps the icon, and "Open Captain" brings the window back.
5. Quit exits the app.

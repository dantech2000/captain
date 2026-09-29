# ADR 0006: The menu bar icon with tray-icon and muda

- Status: Accepted
- Date: 2026-09-29

## Context

M10 puts a Captain icon in the macOS menu bar and the Windows notification area (see [0009](../features/0009-menu-bar.md)). The icon shows the engine state, and its menu has status lines, container and project actions, and Quit. GPUI has no API for a status item or a tray icon. It owns the platform run loop: on macOS it calls `[NSApp run]` in `Platform::run` and calls our launch callback from `applicationDidFinishLaunching:`.

## Decision

- Use the `tray-icon` (0.25) and `muda` (0.20) crates from the Tauri project. `tray-icon` makes an `NSStatusItem` on macOS and a `Shell_NotifyIcon` on Windows. `muda` builds the native menu. Both are small, safe Rust APIs with no event loop of their own, so they run on the loop that GPUI already drives. `tray-icon` re-exports `muda`; Captain names `muda` directly with `default-features = false` so both use one copy.
- Threads: `tray-icon` must build the icon on the main thread after the app has launched. `captain-app` calls `tray::start` at the end of GPUI's launch callback, which runs on the main thread inside `applicationDidFinishLaunching:`. The `TrayIcon` handle is not `Send`, so it lives in a GPUI entity held by a global.
- Menu clicks: `muda` calls a handler on the main thread when the user picks an item. The handler, set with `MenuEvent::set_event_handler`, sends the item id into a `futures` channel. A GPUI foreground task waits on that channel and runs the command inside `cx.update`, where it can reach the workspace and the window. Nothing polls. `tray-icon`'s own click and hover events get an empty handler, so they do not pile up in its unbounded channel.
- Item ids: each rebuild gives the items fresh ids (`captain-tray-<n>`) and keeps a table from id to command. The menu model is plain data (`TrayItem`, `TrayCommand`) built from a small snapshot of the workspace, so it has unit tests without a menu bar.
- Updates: the tray observes the workspace entity. Stats samples notify it many times a second, so on each notify it builds the snapshot (engine state, container ids, names, states, projects, and published ports) and compares it with the one the menu shows. Only a change schedules a rebuild, 200 ms later, which coalesces bursts such as `docker compose up`. The icon image changes only when the engine state changes.
- The icon: a ship's wheel drawn in code into a 36 by 36 RGBA buffer (18 points at 2x) with 4 by 4 supersampling. On macOS it is a template image, so the menu bar tints it for light and dark. A starting engine dims the wheel; a stopped engine dims it and adds a slash. Windows has no template images, so the wheel is white there, which suits the default dark taskbar.
- The window lifecycle: the app, not the window, owns the `Workspace` entity. `AppShell::with_workspace` builds a window around it. Closing the main window keeps Captain and its engine connection running; "Open Captain" and a click on the Dock icon (`Application::on_reopen`) open a new window on the same workspace, or bring the open one to the front. The quit mode is `QuitMode::Explicit` on every platform, and a window-closed handler quits when the last window closes and there is no tray icon. So if the platform refuses the icon, closing the window still quits.
- Linux: the tray is compiled only on macOS and Windows (`cfg(any(target_os = "macos", target_os = "windows"))`). On Linux, `tray-icon` needs GTK and libappindicator, and it needs a GTK main loop running on the thread that owns the icon. GPUI's Linux platform runs its own Wayland or X11 loop, not GTK's. Linux keeps the old behavior: closing the last window quits.

## Consequences

- Two more dependencies on macOS and Windows. They are maintained with Tauri and used widely.
- The workspace lives for the whole run. A window that closes and opens again keeps the connection, the container list, and the selected page.
- Rebuilding the whole menu on a change is simple, but an open menu can change under the pointer. Rebuilds happen only on real changes, so this is rare.
- Linux has no tray. A later change can add one with a GTK loop on a helper thread, or with a StatusNotifierItem over D-Bus.
- The icon, the menu, and the window lifecycle need a manual check in the running app; unit tests cover the snapshot, the menu model, and the icon drawing.

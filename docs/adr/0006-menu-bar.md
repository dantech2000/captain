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

## Update: the popover (M26), then back to the native menu

- M26 made a left click open a GPUI popover (`WindowKind::PopUp`, a non-activating `NSPanel` on macOS) and kept the `muda` menu on a right click.
- The `traymenu` change removed the popover. The user wanted the standard OS menu, matching the system's style like Rancher Desktop's menu. tray-icon's default opens the menu on a left click too, so both clicks open the same `muda` menu. The `TrayIconEvent` handler now drops every icon event, so they do not pile up in tray-icon's channel.
- Every popover action is a menu item now: the engine line with CPUs and memory, the worst problem with its fixes (Raise Memory, Show Logs in a Window, Stop or Restart, the Diagnostics fix), Open Ports, a Kubernetes check item, and Stop All Containers. See [feature 0009](../features/0009-menu-bar.md).
- Rebuilds stay cheap: the snapshot holds only what the menu shows. The engine line's memory changes with each stats sample, so when only that line differs, its text changes in place (`IconMenuItem::set_text`) and the menu is not built again. Other changes still rebuild the menu 200 ms later.
- Status lights: colored circle characters at the start of the item text, because a status item's menu showed no item images on current macOS (drawn images and the system status dots both failed when tested), and muda has no text color. See feature 0009.
- The icon is still the ship's wheel, with a notch dot when a container needs attention. See [feature 0012](../features/0012-icon-theme.md).

## Update: the stop-light dot

- The wheel now carries a colored status dot: green, amber, red, or none while stopped. Setting `menu_bar_status_dot` turns it off and brings back the plain template icon.
- A template image cannot hold a colored dot, and tray-icon marks the whole image as a template or not. On macOS, `status_dot.rs` uses `TrayIcon::ns_status_item` to replace the button's image with an `NSImage` drawing handler that draws the wheel in `labelColor` and the dot in a system color. AppKit runs the handler with the menu bar's appearance, so the wheel still follows light and dark menu bars. Windows paints the dot into the RGBA image.
- `objc2-app-kit` and `block2` gain the features for this, in the versions tray-icon already uses. All calls are safe Rust; `unsafe_code` stays forbidden. See [feature 0009](../features/0009-menu-bar.md).
- The floating log window and the Dock badge stay. See [feature 0032](../features/0032-menu-bar-popover.md).

## Update: the app menus

- Captain now has the standard macOS menus: Captain, File, Edit, View, Window, and Help. See [feature 0042](../features/0042-app-menus.md).
- File > Close Window (⌘W) closes the window in front. Closing the main window this way keeps Captain running while the menu bar icon is up, as the red close button does.
- ⌘W has two bindings. Close Window has no key context; the terminal panel's Close Tab is bound in the panel's context. GPUI prefers the deeper context, so ⌘W closes a terminal tab while the panel has focus, and the window everywhere else.

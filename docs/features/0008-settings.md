# Feature 0008: settings

- Milestone: M7 (settings and contexts)
- Status: Done for this scope; M7 goes on with [0026](0026-contexts-and-remote-hosts.md). Since M30 the one-page Settings ([0037](0037-settings-page.md)) replaced the cards described here, and themes ([0028](0028-themes.md)) replaced the accent color.
- Builds on: [0002](0002-v2-interface.md), which added the empty Settings page
- Decision: [ADR 0004](../adr/0004-settings-file.md) (the settings file)

## Goal

Let the user pick light or dark mode and an accent color, see which engine Captain uses, and switch to another engine without a restart. Captain keeps these choices in a settings file.

## In scope

- Settings model in `captain-core`: `Settings` with a `version`, `appearance` (System, Light, Dark), `accent` (Blue, Purple, Orange, Teal, Graphite), and `engine_endpoint`, an optional `DOCKER_HOST`-style URL. It loads and saves JSON at a path the app gives it. A save writes a temporary file and renames it.
- Settings file: `Captain/settings.json` in the config directory, for example `~/Library/Application Support/Captain/settings.json` on macOS. A missing file gives the defaults. A file that cannot be read gives the defaults and a warning in the log.
- Settings global in `captain-ui`: `captain_ui::settings_init` installs the loaded settings. A change saves at once and redraws every window.
- Appearance card: a System / Light / Dark segmented control, and five accent swatches. System follows the OS, as before. Light and Dark force the mode. The accent colors buttons, links, and selections in both modes. Project colors do not change with the accent.
- Engine card: the connection state with a Reconnect button, the engine version and API version, the OS and architecture, CPUs and memory, and the endpoint in use: Automatic (discovery) or the saved custom endpoint, with "Use automatic" to go back to discovery.
- Switch engine card: the engines that exist on this machine (`DOCKER_HOST`, the current context, and known sockets that exist, from `captain_docker::candidates`), each with a Use button. A Rescan button looks again. A custom endpoint field checks the URL with `Endpoint::parse` and shows a red hint when Captain cannot use it. "Use this engine" saves the endpoint and reconnects.
- Reconnect: `Workspace::reconnect` drops the engine, its tasks, and the loaded containers, then connects again. The app's connector uses the saved endpoint first, then discovery.
- Retry: the containers error state has a Retry button that reconnects.
- About card: the Captain version, the license, and links to the source code and the issue tracker.

## Out of scope

- Remote hosts over SSH (`ssh://` endpoints), and listing, creating, and switching Docker CLI contexts. [0026](0026-contexts-and-remote-hosts.md) adds them.
- More settings, for example the stats history length, which stays fixed.
- A settings window separate from the main window.

## Verification

1. Run `cargo test -p captain-core settings`. The tests load, save, and parse settings files in a temp directory.
2. Run `cargo test -p captain-docker discovery`. The tests cover the list of detected engines.
3. Start Captain and open Settings. The Engine card shows Connected, the engine version, the platform, and the resources.
4. Click Dark, then Light, then System. The window changes mode at once. System follows the OS appearance again.
5. Click each accent swatch. Buttons and links change color at once, in light and in dark mode.
6. Quit and start Captain. The appearance and accent stay. `settings.json` holds them.
7. In the custom endpoint field, type `ftp://host` and click "Use this engine". A red hint shows, and nothing changes.
8. Type the path of a socket that exists, for example `unix:///var/run/docker.sock`, and click "Use this engine". The state goes to Connecting, then Connected, and the Endpoint row shows the custom endpoint.
9. Type `unix:///nonexistent.sock` and use it. The Containers page shows the error state. Click "Use automatic" in Settings. Captain connects again.
10. Stop the engine. The Containers page shows the error state. Start the engine and click Retry. The containers come back.

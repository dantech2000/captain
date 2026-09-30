# Feature 0037: The one-page Settings

- Milestone: M30 (with the settings file)
- Status: Implemented; checked with unit tests. A check by hand in the app is left.
- Builds on: [0015](0015-app-behavior.md), [0024](0024-kubernetes.md), [0026](0026-contexts-and-remote-hosts.md), [0027](0027-v3-interface.md), [0035](0035-command-line-tools.md), and the settings file work (setfile)

## Goal

Settings fits on one screen. At 1440×900 the page does not scroll. It shows the choices most people change, and sends everything else to the settings file. The designs are the canvas boards "SettingsSimple" (the page) and "SettingsTerminalSheet" (the terminal sheet).

Before, the page stacked nine cards: Appearance, Behavior, Captain Engine, Docker daemon, Kubernetes, Administrative access, Connection, Switch engine, Command-line tools, and About.

## In scope

A centered column, about 760 px wide, with a 26 px "Settings" title. Each section is a card with a 150 px label column.

1. **Appearance.** The three themes as small buttons with a strip of their colors (`theme_card.rs`), and System, Light, or Dark.
2. **Engine.** A menu button with the engine in use and its state: "Captain Engine · Running", or the other engine's name ("Rancher Desktop · Connected"). The menu lists Captain Engine, the engines found on this computer, the Docker contexts, and "Add a remote host…". That item opens the engines sheet: the endpoint (Automatic or Custom, with Use automatic), the found engines, the contexts with Use, Make default, and Create or Update context, Rescan, and a remote host field (`ssh://user@host` and the other URL kinds).
   - With Captain Engine: a Resources row with CPU, memory, and disk steppers (the limits of `HostResources`), and the note "Changes apply the next time the engine starts." with this computer's memory and the free space on the engine's disk.
   - With another engine: the endpoint, the Docker version, and Reconnect.
   - "…" menu: Start, Stop, Restart, Bring data from another engine…, Show engine files (the Lima instance folder), and Reset engine… (the old reset dialog). With another engine: Reconnect and Bring data from another engine….
3. **Kubernetes.** One switch. When it is on: the version picker, Apply now, Reset…, a status line, and the memory advice when the engine has less than 8 GB.
4. **Startup.** "Open Captain at login" (the login item) and "Show Captain in the menu bar".
5. **Terminal.** One line and one button. When all three steps are done: "docker, Compose, and Buildx in your terminal use Captain Engine ✓" and a Details link. Otherwise an amber line that names the first thing left, such as "Your terminal's docker still comes from Rancher Desktop.", and Set up…. `cli_tools::SetupSteps` decides.
   The sheet (a gpui-kit dialog) is a checklist with "N of 3 done":
   1. **Link the tools**, with Relink (Link before the first install).
   2. **Put ~/.captain/bin first on your PATH.** For a missing line in a shell file Captain may write: "Add to ~/.zshrc". It turns on the Automatic block of 0035 and installs it. For a file that links into `/nix/store` (home-manager): the line `home.sessionPath = [ "$HOME/.captain/bin" ];` with Copy, and "run home-manager switch". [`home.sessionPath`](https://nix-community.github.io/home-manager/options.xhtml#opt-home.sessionPath) prepends its folders to `PATH` ([home-environment.nix](https://github.com/nix-community/home-manager/blob/master/modules/home-environment.nix)). A link shows the plain shell line too. For a chezmoi file or another link: the reason and the line with Copy.
   3. **Make docker use Captain Engine**: Use Captain Engine (0035's dialog).
   4. Optional: **let tools that use /var/run/docker.sock reach Captain Engine**. This is the old Administrative access card. It asks for a password.
   "Where each tool comes from now" opens the list of tools and their source. Check again reads everything again.
6. **Everything else.** A dashed row: "Open settings file" and "All options". All options opens a sheet with a search field and every key of the settings file with its value.
7. **About line.** "Captain 0.1.0 · Lima 2.0.3 · Docker 28.5 · Licenses". The Lima version comes from the instance's `lima-version` file. Licenses opens the bundle's `licenses` folder, or the repository in a `cargo run` build.

Every new control has a status bar sentence (`HelpExt`).

### Settings that moved to the file

The page no longer shows these. The code that applies them stays.

| Setting | Key |
|---|---|
| Registry mirrors, custom `daemon.json` keys, and the TCP socket (the Docker daemon card) | `engine_daemon` |
| Kubernetes port | `kubernetes.port` |
| Traefik | `kubernetes.traefik` |
| Stop the engine when Captain quits | `stop_engine_on_quit` |
| Start in the background | `start_in_background` |
| PATH Automatic or Manual | `command_line_tools.path` ("Add to ~/.zshrc" sets it to `automatic`) |

## Out of scope

- The settings file itself: comments, overrides only, the schema, the reference entries, live reload, and the real `open_settings_file`. The settings file work (setfile) builds them. Until it lands:
  - `settings_file::open_settings_file` writes the file if it is missing and opens it with `open -t` (macOS), `xdg-open` (Linux), or `start` (Windows).
  - All options lists the keys of the current settings (`options_entries::placeholder_entries`), without descriptions.
- The source code and issues links of the old About card.

## Notes

- The menus use gpui-kit's `DropdownMenu` on a `Button` ([gpui-component `menu/dropdown_menu.rs`](https://github.com/longbridge/gpui-component/blob/main/crates/ui/src/menu/dropdown_menu.rs)). The sheets are `WindowExt::open_dialog`. Their content is an entity that observes the Settings view, so it redraws when a background read ends.
- `EngineHost` has three new methods with `None` defaults: `files_dir`, `runtime_version`, and `free_disk`. `LimaHost` implements them; `free_disk` runs `df` in the background.
- The PATH block's comment now says to set `command_line_tools.path` to `manual` in the settings file, because the page has no PATH control.

## Verification

1. Run `cargo test -p captain-core setup` and `cargo test -p captain-ui options_entries`.
2. Open Captain at 1440×900 and open Settings. The page does not scroll. Hover each control; the status bar shows its sentence.
3. Click each theme and each mode. Step the CPUs, memory, and disk. The note shows the Mac's memory and free disk.
4. Open the engine menu. Pick another engine, then Captain Engine. Open "Add a remote host…", type `ssh://user@host`, and click Connect.
5. Open "…". Show engine files opens `~/.captain/lima/captain` in the Finder.
6. Turn Kubernetes on. The version picker and status line show.
7. With a home-manager `~/.zshrc`, click Set up…. Step 2 shows the `home.sessionPath` line with Copy. With a plain `~/.zshrc` in a temp `HOME`, "Add to ~/.zshrc" adds the `# >>> captain >>>` block.
8. Click Open settings file. The file opens in the text editor. Click All options and search for `port`.

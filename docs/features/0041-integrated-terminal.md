# Feature 0041: Integrated terminal

- Milestone: M34
- Status: Built in code (2026-10-01). Not checked by hand in the app yet.
- Design: a bottom panel like Zed's terminal panel

## Goal

Give Captain its own terminal for shells on this computer, so Captain never opens Terminal.app or another app for a shell. Each shell's `docker` uses the engine Captain shows, without the user's docker context or shell files.

## Why

- The Project page's **Terminal** button opened Terminal.app in the project folder. Captain cannot pass environment variables through `open -a Terminal`, so `docker` in that window used the user's current docker context, which is often another engine.
- Captain already has a terminal emulator ([ADR 0007](../adr/0007-terminal-emulator.md)) and a terminal view for container exec ([0011](0011-terminal.md)). A local shell needs only a PTY and a place in the window.

## Scope

### The panel

- A panel under the page area. It spans the page and the details panel, not the rail or the projects list.
- ⌃` shows or hides it everywhere, as in Zed ([default keymap](https://github.com/zed-industries/zed/blob/main/assets/keymaps/default-macos.json): `"ctrl-`": "terminal_panel::Toggle"`). The ⌘K palette has a **Toggle terminal** row. The rail has a terminal button above Diagnostics, with the tooltip "Terminal ⌃`".
- Drag the top edge to resize it, from 120 px to the window height less 240 px for the page. Double-click the edge to go back to 280 px. The panel works like the inspector's left edge (`inspector/resize_handle.rs`).
- Tabs: the title the shell sets, else the folder name (`~` for the home folder). Each tab has a close button. **+** opens a tab in the folder of the project that the Project page shows, else in the home folder. A chevron button hides the panel.
- Hiding the panel keeps its tabs and their shells. The open state and the height live in the workspace, like `details_hidden`. They do not outlive the app.
- The first show opens a tab. Closing the last tab hides the panel.
- The Project page's **Terminal** button opens a tab in the project's working folder and shows the panel.

### The session

- The user's login shell: `$SHELL -l` on macOS and Linux (`/bin/zsh` on macOS when `SHELL` is empty, else `/bin/sh`). On Windows, Windows PowerShell (`-NoLogo`), or `%COMSPEC%` when PowerShell is missing.
- A dim first line in each new tab: `docker → Captain Engine (unix:///…/docker.sock)`, or the name of the other engine (`engine_name`), or "your docker context" when Captain is not connected.
- Resize: the grid fits whole cells; the PTY resizes after the size holds still for 150 ms (`TIOCSWINSZ`, or `ResizePseudoConsole` on Windows).
- Exit: the grid gets `[Process exited with code N]`, and a bar offers **Restart** and **Close tab**.
- Close: closing a tab, restarting it, or quitting sends `SIGHUP` to the shell's process group and to the PTY's foreground group, as a closed terminal window does. A shell that is still there after 1 second gets `SIGKILL`. On Windows the shell is terminated, and closing the pseudoconsole ends the programs attached to it.

### The environment

Each tab gets Captain's own environment with these changes (`captain_core::host_shell`):

| Variable | Value |
|----------|-------|
| `DOCKER_HOST` | The endpoint Captain is connected to, for example `unix:///Users/me/.captain/lima/captain/sock/docker.sock`. While Captain is not connected, Captain Engine's endpoint when the settings choose it, else unchanged. |
| `DOCKER_CONTEXT` | Removed when `DOCKER_HOST` is set. The docker CLI prefers `DOCKER_CONTEXT` over `DOCKER_HOST` ([docker CLI environment variables](https://docs.docker.com/reference/cli/docker/#environment-variables)). |
| `PATH` | Captain's tool folder first, once: `~/.captain/bin` when its `docker` link works, else `Captain.app/Contents/Resources/bin`. |
| `DOCKER_CONFIG` | `~/.captain/docker`, the folder that Captain's own docker runs use (`DockerCli` in captain-docker), when it exists and the user's `config.json` does not list `~/.captain/cli-plugins`. It lists the bundled Compose and Buildx first. Unchanged otherwise. |
| `KUBECONFIG` | Unchanged. |
| `TERM`, `COLORTERM`, `TERM_PROGRAM` | `xterm-256color`, `truecolor`, `Captain`. |

### Keys

- While a grid has focus, Ctrl-letter keys go to the shell. A `NoAction` binding in the terminal's key context hides the app's Ctrl bindings there: Ctrl-K (palette), Ctrl-B (sidebar), Ctrl-N (New sheet), Ctrl-Q (Quit), Ctrl-1 to Ctrl-9 and Ctrl-, (pages). The inspector's Terminal tab gets the same change.
- On macOS, ⌘K, ⌘B, ⌘N, ⌘1 to ⌘9, and ⌘, still reach Captain. ⌘C and ⌘V copy and paste. ⌘T opens a tab and ⌘W closes the tab while the panel has focus.
- On Linux and Windows, Captain's shortcuts use Ctrl, so they conflict with the shell. The shell wins inside the terminal: click outside the terminal, or use ⌃` to hide it, to use them. Copy and paste are Ctrl-Shift-C and Ctrl-Shift-V. Ctrl-Shift-T opens a tab and Ctrl-Shift-W closes one, because Ctrl-T and Ctrl-W belong to the shell.
- ⌃` toggles the panel from inside the terminal too.

## Design

- `captain-ui/src/terminal/` is the view that both terminals share: the grid, keys, selection, and `TerminalView`. A `TerminalSource` starts a session and returns an `ExecSession` (input, raw output, a resizer, and the exit code). `ExecSource` runs `docker exec` through the engine. `LocalSource` runs a shell in a PTY.
- `inspector/terminal/` keeps only the Terminal tab's header and the "Container is not running" body around a `TerminalView`.
- `terminal_panel/` is the panel: tabs, the tab strip, the resize edge, and the keys.
- `captain-terminal/src/pty.rs` opens the PTY and starts the shell. `terminal/pty_session.rs` in captain-ui turns it into an `ExecSession` with three threads: one reads output, one writes input, and one waits for the exit.
- `captain-core/src/host_shell/` builds the environment and picks the shell program. The environment builder is pure and has a test.

### The PTY layer: `portable-pty`

Two options were checked, both on their published sources.

- **`alacritty_terminal::tty`** (0.26, already a dependency; [docs](https://docs.rs/alacritty_terminal/0.26.0/alacritty_terminal/tty/index.html), [unix.rs](https://github.com/alacritty/alacritty/blob/master/alacritty_terminal/src/tty/unix.rs)). It supports ConPTY on Windows. But it sets the PTY to non-blocking and expects its `polling` event loop, which owns the `Term` behind a mutex; Captain's `Emulator` trait owns the `Term`. Its resize calls `std::process::exit(1)` when `TIOCSWINSZ` fails, which would end Captain. Its `Drop` blocks on `child.wait()`.
- **`portable-pty`** 0.9 (wezterm, MIT; [docs](https://docs.rs/portable-pty/0.9.0/portable_pty/), [source](https://github.com/wezterm/wezterm/tree/main/pty)). It gives a blocking reader (`try_clone_reader`) and writer (`take_writer`) for plain threads, `resize` that returns an error, a child to `wait` on, and a clonable killer. It uses ConPTY on Windows ([Creating a pseudoconsole session](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session), [ResizePseudoConsole](https://learn.microsoft.com/en-us/windows/console/resizepseudoconsole)). On Unix the child calls `setsid` and takes the PTY as its controlling terminal.

Captain uses `portable-pty`. One detail shapes the close: dropping its writer types a newline and end of file. Captain hangs up the shell first and drops the writer only after the shell exited, so a half-typed command never runs.

## Out of scope

- Remembering tabs, their folders, or the panel state across launches.
- Split panes, search in the scrollback, and links in the output.
- A font size setting for the terminal.
- Moving the panel to the side, or a terminal in its own window.

## Notes

- A login shell reads the user's startup files. They can reorder `PATH` (macOS `path_helper` in `/etc/zprofile`) or set `DOCKER_HOST` or `DOCKER_CONTEXT` again. The banner shows what Captain set, not what the shell files did after.
- When `DOCKER_CONFIG` points at `~/.captain/docker`, `docker login` in the panel writes there. Captain rewrites that file from the user's config at its next start, so the login does not stay. Setting up the command-line tools (Settings > Terminal) avoids this: the user's own config then lists Captain's plugins and `DOCKER_CONFIG` stays unchanged.
- A shell's environment is fixed when the tab starts. **Restart** reuses it. A new tab follows a change of engine.

## Verification

Automated:

- `captain_core::host_shell::env` test: `DOCKER_HOST`, `DOCKER_CONTEXT` removed, the tool folder first on `PATH` once, `DOCKER_CONFIG`, no `KUBECONFIG`.
- `terminal_panel::tab_title` test: the shell's title, else the folder name, else `~`.
- `terminal_panel::resize_edge` test: the height follows the mouse within its limits.
- `captain-terminal` PTY test (Unix): `/bin/sh -c 'echo hi$GREETING'` with an extra variable prints `hi-there` and exits with 0.

By hand, with Captain Engine running:

1. Press ⌃`. The panel opens under the page with one tab named `~`. The first line is dim and reads `docker → Captain Engine (unix:///…/docker.sock)`.
2. Run `echo $DOCKER_HOST $DOCKER_CONTEXT`, `which docker`, and `docker ps`. The endpoint matches the banner, `DOCKER_CONTEXT` is empty, and `docker ps` lists Captain Engine's containers.
3. Open a project page and click **Terminal**. A tab named after the project folder opens there; `pwd` agrees.
4. Drag the panel's top edge. `stty size` changes after you let go. Double-click the edge: the height goes back.
5. Press Ctrl-K, Ctrl-B, Ctrl-N, Ctrl-1 in the shell: each goes to the shell (Ctrl-K cuts the line). ⌘K, ⌘B, and ⌘1 still work on macOS.
6. ⌘T opens a tab, ⌘W closes it. **+** opens a tab in the project's folder on a project page, else in home.
7. Run `vim`, then close its tab. `ps -ef | grep vim` shows nothing.
8. Run `exit 3`. The grid shows `[Process exited with code 3]` and the bar shows **Restart** and **Close tab**. Restart opens a new shell in the same folder.
9. Hide the panel with ⌃` while `top` runs, then show it again. The same tab still runs `top`.
10. Quit Captain with a shell open. No shell from the panel is left (`ps -ef | grep -- "-zsh"` shows only other terminals' shells).

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
- Output: at most 1 MiB of output waits for the view. Past that the reader stops reading, so a program that writes fast, such as `yes`, blocks in the PTY. The view feeds at most 256 KiB to the emulator per repaint, then rests 8 ms, so the window keeps drawing and taking input.
- Close: closing a tab, restarting it, or quitting ends the session on a thread of its own, apart from the input writer. It notes the shell's process group and the PTY's foreground group first, then sends `SIGHUP` to both, as a closed terminal window does. It waits until the shell has exited and both groups are gone. A group that is still there after 1 second gets `SIGKILL`, and so does the shell; then the end waits up to 500 ms more. A foreground program that ignores `SIGHUP` thus dies even when the shell exits first. A shell that already exited gets no signals. On Windows the shell is terminated, and closing the pseudoconsole ends the programs attached to it.
- After the end, the reader stops and writes fail, even a write that waits for room in a full PTY. On Unix both use non-blocking copies of the PTY and wait in 100 ms steps (`select` on macOS, where `poll` does not work on terminals). On Windows, closing the pseudoconsole breaks both pipes.
- Quit starts the end of every tab at once and waits up to 2 seconds in total for them. GPUI gives the future of a quit handler only 200 ms, so the handler itself waits.

### The environment

Each tab gets Captain's own environment with these changes (`captain_core::host_shell`):

| Variable | Value |
|----------|-------|
| `DOCKER_HOST` | The endpoint Captain is connected to, for example `unix:///Users/me/.captain/lima/captain/sock/docker.sock`. An `http://` endpoint becomes `tcp://`, because the docker CLI rejects `http://` ([hosts.go](https://github.com/docker/cli/blob/master/opts/hosts.go)); Compose runs use the same `captain_core::docker_host::cli_host`. While Captain is not connected, Captain Engine's endpoint when the settings choose it, else unchanged. |
| `DOCKER_CONTEXT` | Removed when `DOCKER_HOST` is set. The docker CLI prefers `DOCKER_CONTEXT` over `DOCKER_HOST` ([docker CLI environment variables](https://docs.docker.com/reference/cli/docker/#environment-variables)). |
| `DOCKER_TLS`, `DOCKER_TLS_VERIFY`, `DOCKER_CERT_PATH` | Removed when `DOCKER_HOST` is set. The docker CLI turns TLS on from them whatever the host is ([options.go](https://github.com/docker/cli/blob/master/cli/flags/options.go)). Captain talks plain HTTP to every engine (a Unix socket, a named pipe, an SSH tunnel's socket, or TCP), so it never sets them. |
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
- `captain-terminal/src/pty.rs` opens the PTY and starts the shell. `pty/end.rs` ends it (`PtyControl::terminate`), and `pty/unix_io.rs` has the non-blocking reader and writer. `terminal/pty_session.rs` in captain-ui turns it into an `ExecSession` with three threads: one reads output, one writes input, and one waits for the exit. A fourth thread ends the session (`Closing`), and `terminal/output_budget.rs` caps the queued output.
- `captain-core/src/host_shell/` builds the environment and picks the shell program. The environment builder is pure and has a test.

### The PTY layer: `portable-pty`

Two options were checked, both on their published sources.

- **`alacritty_terminal::tty`** (0.26, already a dependency; [docs](https://docs.rs/alacritty_terminal/0.26.0/alacritty_terminal/tty/index.html), [unix.rs](https://github.com/alacritty/alacritty/blob/master/alacritty_terminal/src/tty/unix.rs)). It supports ConPTY on Windows. But it sets the PTY to non-blocking and expects its `polling` event loop, which owns the `Term` behind a mutex; Captain's `Emulator` trait owns the `Term`. Its resize calls `std::process::exit(1)` when `TIOCSWINSZ` fails, which would end Captain. Its `Drop` blocks on `child.wait()`.
- **`portable-pty`** 0.9 (wezterm, MIT; [docs](https://docs.rs/portable-pty/0.9.0/portable_pty/), [source](https://github.com/wezterm/wezterm/tree/main/pty)). It gives a blocking reader (`try_clone_reader`) and writer (`take_writer`) for plain threads, `resize` that returns an error, a child to `wait` on, and a clonable killer. It uses ConPTY on Windows ([Creating a pseudoconsole session](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session), [ResizePseudoConsole](https://learn.microsoft.com/en-us/windows/console/resizepseudoconsole)). On Unix the child calls `setsid` and takes the PTY as its controlling terminal.

Captain uses `portable-pty`. Its Unix writer types a newline and end of file when dropped, and both its reader and writer block. So on Unix Captain does not take them: it copies the master's descriptor with `filedescriptor` (a portable-pty dependency, which keeps the crate free of `unsafe`) and reads and writes it without blocking.

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

- `captain_core::host_shell::env` tests: `DOCKER_HOST`, `DOCKER_CONTEXT` and the TLS variables removed, the tool folder first on `PATH` once, `DOCKER_CONFIG`, no `KUBECONFIG`; an `http://` engine gives `tcp://`.
- `captain_core::docker_host` test: `http://` becomes `tcp://`; other hosts stay.
- `captain-terminal` end tests (Unix): a foreground job that ignores `SIGHUP` and outlives its shell is gone after `terminate`; a write that waits for room in a full PTY fails after `terminate`.
- `terminal::output_budget` test: a full queue holds the reader until the view takes output; an empty queue takes a large chunk; a view that stops reading lets the reader go.
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
11. Run `python3 -c 'import signal, time; signal.signal(signal.SIGHUP, signal.SIG_IGN); time.sleep(600)'`, then close the tab. After about a second, `ps -ef | grep time.sleep` shows nothing. Do it again and quit Captain instead: the same.
12. Run `yes`. The window still scrolls, takes keys, and closes the tab at once. Captain's memory stays flat in Activity Monitor.

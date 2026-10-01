# Feature 0011: Terminal

- Milestone: M8 (exec terminal)
- Status: In progress
- Design: the Terminal tab of the v2 inspector

## Goal

Open a shell in a running container from the inspector, like `docker exec -it <container> sh`, without a separate terminal app.

## In scope

- Exec API: `ContainerApi::exec(id, ExecSpec)` returns an `ExecSession` with an input sink, a byte stream of output, a resize handle, and a future that resolves with the exit code. The Docker engine uses `create_exec`, an attached `start_exec` with a TTY, and `resize_exec`. Bollard types stay in `captain-docker`.
- Default shell: `/bin/bash` if the container has it, else `/bin/sh`. Captain runs a short probe exec, `/bin/bash -c "exit 0"`, and checks its exit code. The probe needs nothing else in the image (no `which`, no `sh`) and its output never shows. The session reports the shell it picked.
- `TERM=xterm-256color` and `COLORTERM=truecolor` in the exec environment.
- Emulation in `captain-terminal`, behind the `Emulator` trait (see [ADR 0007](../adr/0007-terminal-emulator.md)). The backend is `alacritty_terminal`.
- The tab starts the exec the first time it shows for a container, not on selection. Selecting another container, stopping this one, or closing the inspector ends the session.
- A header with a state dot, the shell and container name (or the title the program sets), the grid size as `cols × rows`, and a Reconnect button once the session ends.
- States: connecting, running, exited with the exit code, failed with the engine's message, and "Container is not running" for a stopped container.
- A grid drawn in the monospace font at 12 px: cell backgrounds, then text runs per row with color, bold, italic, underline, and strikeout, then a block, beam, or underline cursor. An unfocused grid shows a hollow block. The 16 ANSI colors have dark and light variants tuned to Captain's palette.
- Keys: printable text, Enter, Tab, Shift-Tab, Backspace, Escape, arrows (application cursor mode aware), Home, End, Page Up, Page Down, Insert, Delete, F1 to F12, Ctrl and Alt combinations. On macOS, Option types the character it composes.
- Ctrl-K, Ctrl-Q, Ctrl-B, Ctrl-N, Ctrl-1 to Ctrl-9, and Ctrl-, go to the shell while the grid has focus. The app binds them to the palette, Quit, the sidebar, the New sheet, and the pages elsewhere. The Cmd shortcuts still work on macOS. The terminal panel shares this view ([0041](0041-integrated-terminal.md)).
- Copy and paste: Cmd-C and Cmd-V on macOS, Ctrl-Shift-C and Ctrl-Shift-V elsewhere. Paste uses bracketed paste when the program asks for it. ESC and Ctrl-C (`\x03`) are removed from a bracketed paste, as alacritty does, so pasted text cannot end the paste early and run commands ([alacritty event.rs](https://github.com/alacritty/alacritty/blob/master/alacritty/src/event.rs)).
- Mouse: drag to select, double click selects a word, triple click a line.
- Scroll wheel: scrolls 10,000 lines of history. On the alternate screen (`less`, `vim`) it sends arrow keys. Typing jumps back to the live screen.
- Resize: the grid fits whole cells into the pane. The emulator resizes at once; the exec resizes after the size holds still for 150 ms.

## Out of scope

- Opening the terminal in its own window.
- Mouse reporting to programs, OSC 52 clipboard access, and color queries.
- IME composition and dead keys.
- Combining characters (accents stored as separate code points).
- Several terminals per container, and exec as another user or in another directory.
- `libghostty-vt`. It waits for Zig in the toolchain.

## Notes

- Ending a session closes the connection, so the shell reads end of file and exits. A program that ignores end of file, such as a running `top`, keeps running in the container until it exits by itself.
- The inspector is 400 px wide, so the grid is about 45 columns.

## Verification

1. `docker run -d --name captain-term nginx:alpine`, select it, and open Terminal. A prompt appears within a second, and the header shows `/bin/sh · captain-term`.
2. `docker run -d --name captain-bash debian:stable-slim sleep infinity` gives `/bin/bash` in the header.
3. `ls --color=auto /`, `top`, and `vi` draw correctly. Arrow keys move in `vi` and in shell history.
4. Resizing the window changes `cols × rows`, and `stty size` in the shell agrees.
5. Selecting text and pressing Cmd-C (Ctrl-Shift-C) copies it. Cmd-V (Ctrl-Shift-V) pastes a multi-line text without running it line by line in bash.
6. The scroll wheel scrolls back through `seq 1 500` output. Typing jumps back to the prompt.
7. `exit 3` shows the exit code and a Reconnect button. Reconnect opens a new shell.
8. Stopping the container shows "Container is not running". Starting it again and showing the tab opens a new shell.
9. Ctrl-K cuts to the end of the line in bash, and does not open the command palette.
10. `cargo test -p captain-docker --test live_exec -- --ignored` passes.

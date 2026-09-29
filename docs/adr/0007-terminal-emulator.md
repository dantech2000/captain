# ADR 0007: The terminal emulator sits behind a trait

- Status: Accepted
- Date: 2026-09-29

## Context

M8 adds an interactive shell to the container inspector (see [0011](../features/0011-terminal.md)). A shell needs a terminal emulator: a VT parser, a grid with history, and an encoder for keys and pastes.

The roadmap named `libghostty-vt`. It builds with Zig, and Zig is not part of Captain's toolchain yet. Nix and CI would both need it before the first line of terminal code could build. `alacritty_terminal` (0.26) is pure Rust, maintained with Alacritty, and used by Zed's terminal. It builds with the toolchain we already have.

We want the shell now and the option to move to `libghostty-vt` later without rewriting the view.

## Decision

- A new crate, `captain-terminal`, owns emulation. Its `Emulator` trait is the swap point. `captain-ui` depends on this crate and never names a backend type.
- `alacritty_terminal` is the backend today, behind the default `alacritty` Cargo feature. `AlacrittyEmulator` wraps `Term` and the `vte` `Processor`. An `EventListener` collects the window title and the replies `Term` must write back to the program (for example the answer to a cursor position query).
- A `ghostty` feature is declared and empty. When Zig is in the Nix shell and in CI, a `ghostty` module implements `Emulator` on `libghostty-vt`, and `default_emulator` picks it when the feature is on. Building with no backend is a `compile_error!`.
- `default_emulator(cols, rows) -> Box<dyn Emulator>` is the only constructor the UI calls.

### What the trait covers

- Input to the emulator: `feed(bytes)` for program output, and `resize(cols, rows)`.
- Output to the view: `snapshot() -> Screen`. A `Screen` has rows of `Cell`s (a `char`, text and background `Color`, and flags for bold, italic, underline, inverse, dim, strikeout, hidden, wide, wide spacer, and selected), the cursor (row, column, shape, and visibility), the scroll offset, and the history length.
- Colors: our own `Color` enum with `Foreground`, `Background`, `Indexed(u8)`, and `Rgb`. The view maps the defaults and the 16 ANSI colors to the app theme. Palette entries the program redefines with OSC 4, 10, or 11 arrive as `Rgb`.
- History: `scroll(lines)` and `scroll_to_bottom()`.
- Input from the user: `modes()` reports the modes that change encoding (application cursor keys, bracketed paste, newline mode, alternate screen, alternate scroll). `encode_key(KeyInput)` and `encode_paste(text)` have default implementations in `captain-terminal` built on `modes()`, so every backend encodes the same way unless it overrides them. A backend with its own encoder, such as ghostty's, can override both.
- Selection: `select_start(point, kind)`, `select_update(point)`, `select_clear()`, and `selection_text()`. Points are in view coordinates with a left or right cell half. Kinds are simple, word, and line.
- Program requests: `title()` and `take_replies()`.

### What the UI must never touch

- Any `alacritty_terminal` or `vte` type, or a future ghostty type. `captain-ui` has no dependency on either crate.
- The grid storage or history directly. It reads a `Screen` copy per frame.
- Escape sequence encoding. It maps GPUI keystrokes to `KeyInput` and asks the emulator for bytes.

## Consequences

- The shell ships without a new toolchain.
- A snapshot copies the visible cells each frame. For a grid of a few thousand cells this is cheap, and it keeps borrow lifetimes of the backend out of the view.
- Both backends must pass the same tests in `captain-terminal`. When ghostty lands, the tests that feed ANSI sequences and check cells run against both.
- Mouse reporting to programs, OSC 52 clipboard access, and color queries are not in the trait yet. Adding them later changes the trait, not the view's structure.

//! Terminal emulation for Captain's terminals, and the PTY of the terminal panel.
//!
//! Views talk to an [`Emulator`], never to the backend behind it. Only this crate's own
//! types cross that trait, so the backend can change without touching the UI. See
//! docs/adr/0007-terminal-emulator.md.

#[cfg(feature = "alacritty")]
mod alacritty;
mod color;
mod emulator;
mod encode;
mod input;
mod pty;
mod screen;
mod selection;

#[cfg(feature = "alacritty")]
pub use alacritty::AlacrittyEmulator;
pub use color::{Color, Rgb};
pub use emulator::{Emulator, default_emulator};
pub use encode::{encode_key, encode_paste};
pub use input::{InputModes, Key, KeyInput, Modifiers};
pub use pty::{Pty, PtyChild, PtyControl, ShellCommand, spawn as spawn_pty};
pub use screen::{Cell, CellFlags, Cursor, CursorShape, Screen};
pub use selection::{GridPoint, SelectionKind, Side};

use crate::{GridPoint, InputModes, KeyInput, Screen, SelectionKind, encode};

/// A terminal emulator: it parses the bytes a program writes, keeps the screen and its
/// history, and encodes the user's input for the program.
///
/// This trait is the swap point between backends (alacritty_terminal today,
/// libghostty-vt later). Only this crate's own types cross it. See
/// docs/adr/0007-terminal-emulator.md.
pub trait Emulator: Send {
    /// Parses output from the program.
    fn feed(&mut self, bytes: &[u8]);

    /// Changes the grid size. Both values are at least 1.
    fn resize(&mut self, cols: u16, rows: u16);

    /// The grid size as `(cols, rows)`.
    fn size(&self) -> (u16, u16);

    /// A copy of what the view shows now.
    fn snapshot(&self) -> Screen;

    /// Scrolls the view through the history. Positive `lines` go back (up), negative
    /// lines go toward the live screen.
    fn scroll(&mut self, lines: i32);

    /// Jumps back to the live screen.
    fn scroll_to_bottom(&mut self);

    /// The modes that change how input is encoded.
    fn modes(&self) -> InputModes;

    /// The bytes for a key press, or `None` if the key sends nothing.
    fn encode_key(&self, key: KeyInput) -> Option<Vec<u8>> {
        encode::encode_key(&key, self.modes())
    }

    /// The bytes for a paste, bracketed when the program asked for it.
    fn encode_paste(&self, text: &str) -> Vec<u8> {
        encode::encode_paste(text, self.modes())
    }

    /// The window title the program set, if any.
    fn title(&self) -> Option<String>;

    /// Bytes the emulator must send back to the program, for example the answer to a
    /// cursor position query. Taking them empties the queue.
    fn take_replies(&mut self) -> Vec<u8>;

    /// Starts a new selection at `point`, replacing any old one.
    fn select_start(&mut self, point: GridPoint, kind: SelectionKind);

    /// Moves the free end of the selection to `point`.
    fn select_update(&mut self, point: GridPoint);

    fn select_clear(&mut self);

    /// The selected text, or `None` if nothing is selected.
    fn selection_text(&self) -> Option<String>;
}

/// The emulator Captain uses, sized `cols` by `rows`.
#[cfg(feature = "alacritty")]
pub fn default_emulator(cols: u16, rows: u16) -> Box<dyn Emulator> {
    Box::new(crate::AlacrittyEmulator::new(cols, rows))
}

// The `ghostty` feature is declared for libghostty-vt but has no backend yet.
#[cfg(not(feature = "alacritty"))]
compile_error!(
    "captain-terminal needs an emulator backend: enable the `alacritty` feature \
     (the `ghostty` backend is not implemented yet, see docs/adr/0007-terminal-emulator.md)"
);

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use alacritty_terminal::event::{Event, EventListener};

/// Collects the events from `Term` that the emulator reports: the title, and the
/// replies it must write back to the program. The emulator keeps a clone to read them.
#[derive(Clone, Default)]
pub struct Listener {
    state: Arc<Mutex<State>>,
}

#[derive(Default)]
struct State {
    title: Option<String>,
    replies: Vec<u8>,
}

impl Listener {
    pub fn title(&self) -> Option<String> {
        self.lock().title.clone()
    }

    pub fn take_replies(&self) -> Vec<u8> {
        std::mem::take(&mut self.lock().replies)
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl EventListener for Listener {
    fn send_event(&self, event: Event) {
        let mut state = self.lock();
        match event {
            Event::Title(title) => state.title = Some(title),
            Event::ResetTitle => state.title = None,
            Event::PtyWrite(text) => state.replies.extend_from_slice(text.as_bytes()),
            // Clipboard access (OSC 52), color queries, the bell, and redraw hints are
            // not supported yet. Every call to `feed` redraws anyway.
            _ => {}
        }
    }
}

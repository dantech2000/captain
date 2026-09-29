use gpui_kit::*;

use super::ImagesState;

/// A container that the Run dialog started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Started {
    pub id: String,
    /// The name the user gave, or the short ID if the engine picked the name.
    pub name: String,
}

impl ImagesState {
    /// Records a started container, which shows the "Started" notice. Does nothing
    /// if the engine changed since `generation`.
    pub fn set_started(&mut self, started: Started, generation: u64, cx: &mut Context<Self>) {
        if generation != self.generation {
            return;
        }
        self.error = None;
        self.notice = None;
        self.started = Some(started);
        cx.notify();
    }

    pub fn started(&self) -> Option<&Started> {
        self.started.as_ref()
    }

    /// Hides the "Started" notice.
    pub fn dismiss_started(&mut self, cx: &mut Context<Self>) {
        self.started = None;
        cx.notify();
    }
}

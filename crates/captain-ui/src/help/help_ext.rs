use gpui_kit::*;

use super::{Hint, hover_help};

/// The command key as the status bar shows it.
pub const CMD: &str = if cfg!(target_os = "macos") {
    "⌘"
} else {
    "Ctrl"
};

/// Gives an element a help sentence for the status bar. The element needs an id,
/// because GPUI keeps hover state per id.
pub trait HelpExt: StatefulInteractiveElement {
    /// Like [`HelpExt::help`], with the control's shortcut keys, for example
    /// `&[CMD, "K"]`.
    fn help_keys(self, text: impl Into<SharedString>, keys: &'static [&'static str]) -> Self {
        self.help(Hint::with_keys(text, keys))
    }

    /// Shows the hint in the status bar while the mouse is over the element. Call it
    /// once per element: GPUI allows one hover listener.
    fn help(mut self, hint: impl Into<Hint>) -> Self {
        let Some(id) = self.interactivity().element_id.clone() else {
            return self;
        };
        let hint = hint.into();
        self.on_hover(move |hovered, _, cx| {
            let Some(model) = hover_help(cx) else {
                return;
            };
            model.update(cx, |help, cx| {
                let changed = if *hovered {
                    help.enter(id.clone(), hint.clone())
                } else {
                    help.leave(&id)
                };
                if changed {
                    cx.notify();
                }
            });
        })
    }
}

impl<E: StatefulInteractiveElement> HelpExt for E {}

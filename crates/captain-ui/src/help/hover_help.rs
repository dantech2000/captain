use gpui_kit::*;

use super::liveness::is_alive;

/// A help sentence and the shortcut keys of its control, if it has any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    pub text: SharedString,
    pub keys: &'static [&'static str],
}

/// The hints of the controls under the mouse, innermost last, and the elements that
/// set them.
///
/// GPUI can report the new control's hover before the old control's hover-out, so
/// a hover-out removes only the leaving element's hint. That keeps the bar from
/// flickering when the mouse moves between two controls. A control inside another,
/// such as a switch in a Settings row, shows its own hint; leaving it shows the
/// outer one again.
///
/// One mouse move reports its hovers innermost first. When the mouse lands on a
/// button in a card in one move, or moves again after a click, the button and the
/// card enter together; a batch (see [`hover_batch`](super::hover_batch)) keeps the
/// button's hint on top.
#[derive(Debug, Default)]
pub struct HoverHelp {
    hovered: Vec<(ElementId, Hint)>,
    /// The hovers of the mouse move being dispatched, innermost first.
    batch: Option<Vec<(ElementId, Hint)>>,
}

impl Hint {
    /// A sentence for a control with a shortcut, for example `&[CMD, "K"]`.
    pub fn with_keys(text: impl Into<SharedString>, keys: &'static [&'static str]) -> Self {
        Self {
            text: text.into(),
            keys,
        }
    }
}

impl From<&'static str> for Hint {
    fn from(text: &'static str) -> Self {
        Self::with_keys(text, &[])
    }
}

impl From<String> for Hint {
    fn from(text: String) -> Self {
        Self::with_keys(text, &[])
    }
}

impl From<SharedString> for Hint {
    fn from(text: SharedString) -> Self {
        Self::with_keys(text, &[])
    }
}

impl HoverHelp {
    pub fn hint(&self) -> Option<&Hint> {
        self.hovered.last().map(|(_, hint)| hint)
    }

    /// The mouse entered `owner`. True if the hint changed.
    pub fn enter(&mut self, owner: ElementId, hint: Hint) -> bool {
        if let Some(batch) = &mut self.batch {
            batch.retain(|(id, _)| *id != owner);
            batch.push((owner, hint));
            return false;
        }
        let before = self.hint().cloned();
        self.hovered.retain(|(id, _)| *id != owner);
        self.hovered.push((owner, hint));
        self.hint() != before.as_ref()
    }

    /// The mouse left `owner`. True if the hint changed.
    pub fn leave(&mut self, owner: &ElementId) -> bool {
        if let Some(batch) = &mut self.batch {
            batch.retain(|(id, _)| id != owner);
        }
        let before = self.hint().cloned();
        self.hovered.retain(|(id, _)| id != owner);
        self.hint() != before.as_ref()
    }

    /// A mouse move starts: hold its hovers until [`HoverHelp::end_batch`]. True if
    /// an unfinished batch changed the hint.
    pub fn begin_batch(&mut self) -> bool {
        let changed = self.end_batch();
        self.batch = Some(Vec::new());
        changed
    }

    /// The mouse move ends: the controls it entered go on top, outermost first, so
    /// the innermost one shows. True if the hint changed.
    pub fn end_batch(&mut self) -> bool {
        let Some(batch) = self.batch.take() else {
            return false;
        };
        let before = self.hint().cloned();
        for (owner, hint) in batch.into_iter().rev() {
            self.hovered.retain(|(id, _)| *id != owner);
            self.hovered.push((owner, hint));
        }
        self.hint() != before.as_ref()
    }
}

impl HoverHelp {
    /// Drops the hints of elements that left the screen, for example a row that a
    /// cleanup removed while the mouse rested on it. True if the hint changed.
    pub fn forget_gone(&mut self) -> bool {
        if let Some(batch) = &mut self.batch {
            batch.retain(|(id, _)| is_alive(id));
        }
        let before = self.hint().cloned();
        self.hovered.retain(|(id, _)| is_alive(id));
        self.hint() != before.as_ref()
    }
}

struct HoverHelpHandle(Entity<HoverHelp>);

impl Global for HoverHelpHandle {}

/// The app's hover help, created on first use. The main window calls this.
pub fn ensure(cx: &mut App) -> Entity<HoverHelp> {
    if let Some(model) = hover_help(cx) {
        return model;
    }
    let model = cx.new(|_| HoverHelp::default());
    cx.set_global(HoverHelpHandle(model.clone()));
    model
}

/// The app's hover help, once a window with a status bar exists.
pub fn hover_help(cx: &App) -> Option<Entity<HoverHelp>> {
    cx.try_global::<HoverHelpHandle>()
        .map(|handle| handle.0.clone())
}

/// Drops the hint. A click can remove the control under the mouse, and a removed
/// control never reports its hover-out.
pub fn clear(cx: &mut App) {
    if let Some(model) = hover_help(cx) {
        model.update(cx, |help, cx| {
            if let Some(batch) = &mut help.batch {
                batch.clear();
            }
            if !help.hovered.is_empty() {
                help.hovered.clear();
                cx.notify();
            }
        });
    }
}

#[cfg(test)]
mod tests;

use gpui_kit::*;

/// A help sentence and the shortcut keys of its control, if it has any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    pub text: SharedString,
    pub keys: &'static [&'static str],
}

/// The hint of the control under the mouse, and the element that set it.
///
/// GPUI can report the new control's hover before the old control's hover-out, so
/// a hover-out clears the hint only when the leaving element still owns it. That
/// keeps the bar from flickering when the mouse moves between two controls.
#[derive(Debug, Default)]
pub struct HoverHelp {
    current: Option<(ElementId, Hint)>,
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
        self.current.as_ref().map(|(_, hint)| hint)
    }

    /// The mouse entered `owner`. True if the hint changed.
    pub fn enter(&mut self, owner: ElementId, hint: Hint) -> bool {
        let next = Some((owner, hint));
        if self.current == next {
            return false;
        }
        self.current = next;
        true
    }

    /// The mouse left `owner`. True if the hint changed.
    pub fn leave(&mut self, owner: &ElementId) -> bool {
        if self.current.as_ref().is_some_and(|(id, _)| id == owner) {
            self.current = None;
            return true;
        }
        false
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
            if help.current.take().is_some() {
                cx.notify();
            }
        });
    }
}

#[cfg(test)]
mod tests;

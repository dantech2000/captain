use gpui_kit::*;

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
#[derive(Debug, Default)]
pub struct HoverHelp {
    hovered: Vec<(ElementId, Hint)>,
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
        let before = self.hint().cloned();
        self.hovered.retain(|(id, _)| *id != owner);
        self.hovered.push((owner, hint));
        self.hint() != before.as_ref()
    }

    /// The mouse left `owner`. True if the hint changed.
    pub fn leave(&mut self, owner: &ElementId) -> bool {
        let before = self.hint().cloned();
        self.hovered.retain(|(id, _)| id != owner);
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
            if !help.hovered.is_empty() {
                help.hovered.clear();
                cx.notify();
            }
        });
    }
}

#[cfg(test)]
mod tests;

//! The popover's buttons and switches.

use gpui_kit::component::Disableable;
use gpui_kit::component::switch::Switch;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::help::{HelpExt, Hint};
use crate::theme::Palette;

/// How a popover button looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// The action color, for the fix.
    Primary,
    /// A plain button.
    Plain,
    /// Text only, for a way out such as Stop.
    Quiet,
}

/// A button with a text label. `tall` is the footer's size.
pub fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    help: impl Into<Hint>,
    tone: Tone,
    tall: bool,
    palette: &Palette,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let (bg, text, hover) = match tone {
        Tone::Primary => (
            palette.accent,
            palette.on_accent,
            palette.accent.opacity(0.85),
        ),
        Tone::Plain => (palette.button, palette.text, palette.hover),
        Tone::Quiet => (transparent_black(), palette.text2, palette.hover),
    };
    div()
        .id(id)
        .h(px(if tall { 30. } else { 26. }))
        .px(px(if tall { 12. } else { 10. }))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(if tall { 8. } else { 7. }))
        .bg(bg)
        .text_color(text)
        .text_size(px(if tall { 12. } else { 11.5 }))
        .when(tone != Tone::Quiet, |this| {
            this.font_weight(FontWeight::SEMIBOLD)
        })
        .when(tone == Tone::Plain, |this| {
            this.border_1().border_color(palette.sep)
        })
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(move |_, window, cx| on_click(window, cx))
        .child(label.into())
        .help(help)
}

/// A switch with a help sentence. `on_change` gets the new state.
pub fn switch(
    id: impl Into<SharedString>,
    on: bool,
    enabled: bool,
    help: impl Into<Hint>,
    on_change: impl Fn(bool, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let id: SharedString = id.into();
    div()
        .id(SharedString::from(format!("{id}-help")))
        .flex_shrink_0()
        .child(
            Switch::new(id)
                .checked(on)
                .disabled(!enabled)
                .on_click(move |checked, window, cx| on_change(*checked, window, cx)),
        )
        .help(help)
}

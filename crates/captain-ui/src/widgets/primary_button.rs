use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::kit_button::{Look, kit_button};
use crate::help::{HelpExt, Hint};
use crate::theme::Palette;

/// A large button filled with the accent color, for the main action of a screen.
/// A disabled button is gray and ignores clicks. `help` is its status bar sentence.
pub fn primary_button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    help: impl Into<Hint>,
    enabled: bool,
    palette: &Palette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let shape = Shape {
        height: px(34.),
        padding: px(18.),
        text: px(13.),
    };
    build(id, label, help, enabled, palette, shape, on_click)
}

/// A [`primary_button`] for a header or a drawer: lower, with smaller text.
pub fn small_primary_button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    help: impl Into<Hint>,
    enabled: bool,
    palette: &Palette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let shape = Shape {
        height: px(30.),
        padding: px(14.),
        text: px(12.),
    };
    build(id, label, help, enabled, palette, shape, on_click)
}

/// The size of a primary button. The kit button sets its label's text size, so the
/// size cannot come from the caller's div.
struct Shape {
    height: Pixels,
    padding: Pixels,
    text: Pixels,
}

fn build(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    help: impl Into<Hint>,
    enabled: bool,
    palette: &Palette,
    shape: Shape,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let look = Look {
        bg: palette.accent,
        fg: palette.on_accent,
        hover: palette.accent.opacity(0.85),
    };
    let label: SharedString = label.into();
    let (field, text3) = (palette.field, palette.text3);
    kit_button(id, label.clone(), look, enabled, on_click, |button| {
        button
            .flex_grow(1.)
            .h_full()
            .px(shape.padding)
            .rounded(px(9.))
            .font_weight(FontWeight::SEMIBOLD)
            .when(enabled, |button| button.cursor_pointer())
            .when(!enabled, |button| button.bg(field).text_color(text3))
            .child(div().text_size(shape.text).child(label))
    })
    .h(shape.height)
    .flex_shrink_0()
    .help(help)
}

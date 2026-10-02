//! Pieces the New sheet's forms share: labeled fields, the Form and compose.yaml
//! switch, the read-only preview, and the footer with Back and the main button.

use gpui_kit::component::Sizable;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::scroll::{Scrollable, ScrollableElement};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::keys::{FORM_CONTEXT, FocusNext, FocusPrev};
use crate::help::{CMD, HelpExt};
use crate::theme::Palette;
use crate::widgets::{ButtonTone, Segment, inline_error, segmented, text_button};

/// A label above a control, with a red hint under it when the value is not valid.
pub fn field(
    label: impl Into<SharedString>,
    control: impl IntoElement,
    hint: Option<String>,
    palette: &Palette,
) -> Div {
    div()
        .flex()
        .flex_col()
        // In a scrolling step, a field keeps its height; the step scrolls.
        .flex_shrink_0()
        .gap(px(6.))
        .child(
            div()
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text3)
                .child(label.into()),
        )
        .child(control)
        .children(hint.map(|hint| div().text_size(px(11.)).text_color(palette.red).child(hint)))
}

/// A small text field with a help sentence.
pub fn text_input(
    id: impl Into<ElementId>,
    input: &Entity<InputState>,
    help: impl Into<SharedString>,
) -> Stateful<Div> {
    div()
        .id(id)
        .flex_1()
        .min_w_0()
        .child(Input::new(input).small())
        .help(help.into())
}

/// A new text field with `value` in it.
pub fn new_input<T: 'static>(
    placeholder: &'static str,
    value: impl Into<SharedString>,
    window: &mut Window,
    cx: &mut Context<T>,
) -> Entity<InputState> {
    let value = value.into();
    cx.new(|cx| {
        InputState::new(window, cx)
            .placeholder(placeholder)
            .default_value(value)
    })
}

/// The trimmed text of `input`.
pub fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().trim().to_string()
}

/// Wraps a step's form: Tab and Shift Tab move between its fields.
pub fn form_root(id: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .key_context(FORM_CONTEXT)
        .flex()
        .flex_col()
        .gap(px(14.))
        .on_action(|_: &FocusNext, window, cx| window.focus_next(cx))
        .on_action(|_: &FocusPrev, window, cx| window.focus_prev(cx))
}

/// The scrolling area of a step, so a long form fits the window.
pub fn scroll_body(id: &'static str) -> Scrollable<Stateful<Div>> {
    div()
        .id(id)
        .max_h(px(440.))
        .overflow_y_scrollbar()
        .flex()
        .flex_col()
        .gap(px(14.))
        .pr(px(4.))
}

/// Switches between the form and the Compose file it writes.
pub fn view_switch(
    showing_compose: bool,
    on_change: impl Fn(bool, &mut Window, &mut App) + Clone + 'static,
    palette: &Palette,
) -> Div {
    let segment = |label: &'static str, compose: bool, help: &'static str| {
        let on_change = on_change.clone();
        Segment {
            label: label.into(),
            selected: showing_compose == compose,
            help: help.into(),
            on_click: Box::new(move |window, cx| on_change(compose, window, cx)),
        }
    };
    div().flex().child(segmented(
        "new-view",
        vec![
            segment("Form", false, "Show the form."),
            segment(
                "compose.yaml",
                true,
                "Show the Compose file Captain writes from the form. You can edit it after.",
            ),
        ],
        palette,
    ))
}

/// The Compose file as it will be written, read-only, or why the form cannot
/// make it yet.
pub fn compose_preview(text: Result<String, String>, palette: &Palette) -> Div {
    match text {
        Ok(text) => div()
            .flex_shrink_0()
            .p(px(10.))
            .rounded(px(8.))
            .bg(palette.field)
            .font_family(palette.mono())
            .text_size(px(12.))
            .children(text.lines().map(|line| {
                div()
                    .whitespace_nowrap()
                    .min_h(px(16.))
                    .child(line.to_string())
            })),
        Err(error) => inline_error(error, palette),
    }
}

/// Back on the left; the main button on the right. The main button runs with
/// ⌘Return too.
pub fn footer(
    back_help: &'static str,
    on_back: impl Fn(&mut Window, &mut App) + 'static,
    action: (&'static str, String, bool),
    on_action: impl Fn(&mut Window, &mut App) + 'static,
    palette: &Palette,
) -> Div {
    let (label, help, enabled) = action;
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(8.))
        .pt(px(12.))
        .border_t_1()
        .border_color(palette.sep)
        .child(
            text_button(
                "new-back",
                "Back",
                ButtonTone::Accent,
                true,
                palette,
                move |_, window, cx| on_back(window, cx),
            )
            .help(back_help),
        )
        .child(
            text_button(
                "new-create",
                label,
                ButtonTone::Accent,
                enabled,
                palette,
                move |_, window, cx| on_action(window, cx),
            )
            .when(enabled, |button| {
                button.help_keys(help.clone(), &[CMD, "↩"])
            })
            .when(!enabled, |button| button.help(help)),
        )
}

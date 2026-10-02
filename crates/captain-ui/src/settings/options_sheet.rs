//! The All options sheet: every setting with a search field. Until the settings
//! file work (setfile) lands its reference entries, it lists the keys of the
//! current settings; see `options_entries.rs`.

use gpui_kit::component::Sizable;
use gpui_kit::component::WindowExt;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;

use super::options_entries::{OptionEntry, option_entries};
use super::{file_section, store};
use crate::theme::Palette;
use crate::widgets::primary_button;

pub struct OptionsSheet {
    search: Entity<InputState>,
    entries: Vec<OptionEntry>,
    _search: Subscription,
}

pub fn open(window: &mut Window, cx: &mut App) {
    let sheet = cx.new(|cx| {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search the options"));
        let events = cx.subscribe(&search, |_, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        });
        OptionsSheet {
            search,
            entries: option_entries(&store::current(cx)),
            _search: events,
        }
    });
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("All options")
            .w(px(620.))
            .margin_top(px(110.))
            .child(sheet.clone())
    });
}

impl Render for OptionsSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let query = self.search.read(cx).value().to_string();
        let rows: Vec<Div> = self
            .entries
            .iter()
            .filter(|entry| entry.matches(&query))
            .map(|entry| row(entry, &palette))
            .collect();
        let empty = rows.is_empty().then(|| {
            div()
                .p(px(12.))
                .text_color(palette.text3)
                .child(format!("No option matches \"{}\".", query.trim()))
        });
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(div().text_size(px(12.)).text_color(palette.text3).child(
                "Every setting, what it does, its value now, and its default. Change them in settings.json.",
            ))
            .child(Input::new(&self.search).small())
            .child(
                div()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(palette.sep)
                    .overflow_hidden()
                    .child(
                        div()
                            .id("options-sheet-list")
                            .h(px(418.))
                            .overflow_y_scrollbar()
                            .flex()
                            .flex_col()
                            .children(rows)
                            .children(empty),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(file_section::open_button("options-sheet-open", &palette))
                    .child(div().flex_1())
                    .child(primary_button(
                        "options-sheet-done",
                        "Done",
                        "Close the list of options.",
                        true,
                        &palette,
                        |_, window, cx| window.close_dialog(cx),
                    )),
            )
    }
}

fn row(entry: &OptionEntry, palette: &Palette) -> Div {
    div()
        .flex()
        .gap(px(12.))
        .px(px(12.))
        .py(px(7.))
        .border_b_1()
        .border_color(palette.sep)
        .text_size(px(12.))
        .child(
            div()
                .w(px(260.))
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(div().font_family(palette.mono()).child(entry.key.clone()))
                .child(
                    div()
                        .font_family(palette.mono())
                        .text_size(px(11.))
                        .text_color(palette.text3)
                        .truncate()
                        .child(format!("now {} · default {}", entry.value, entry.default)),
                ),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_color(palette.text2)
                .child(entry.description.clone()),
        )
}

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::assistant::{MigrationAssistant, Stage};
use super::{choose_step, review_step, run_step, summary_step};
use crate::theme::Palette;
use crate::widgets::inline_error;

/// The height of the scrolling body, so the dialog keeps its size between steps.
const BODY_HEIGHT: f32 = 420.;

const STEPS: [(&str, &[Stage]); 4] = [
    ("Source", &[Stage::Choose, Stage::Loading]),
    ("Plan", &[Stage::Review]),
    ("Copy", &[Stage::Run]),
    ("Summary", &[Stage::Summary]),
];

impl Render for MigrationAssistant {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let (body, footer) = match self.stage {
            Stage::Choose | Stage::Loading => (
                choose_step::body(self, &palette, cx),
                choose_step::footer(self, &palette),
            ),
            Stage::Review => (
                review_step::body(self, &palette, cx),
                review_step::footer(self, &palette, cx),
            ),
            Stage::Run => (
                run_step::body(self, &palette, cx),
                run_step::footer(self, &palette, cx),
            ),
            Stage::Summary => (
                summary_step::body(self, &palette, cx),
                summary_step::footer(self, &palette, cx),
            ),
        };
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(steps(self.stage, &palette))
            .child(
                div()
                    .id("migration-body")
                    .h(px(BODY_HEIGHT))
                    .overflow_y_scrollbar()
                    .pr(px(4.))
                    .child(body),
            )
            .children(
                self.error
                    .clone()
                    .map(|error| inline_error(error, &palette)),
            )
            .child(footer)
    }
}

/// The four steps, with the current one in the accent color.
fn steps(stage: Stage, palette: &Palette) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .text_size(px(11.))
        .children(STEPS.iter().enumerate().map(|(ix, (label, stages))| {
            let current = stages.contains(&stage);
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .when(ix > 0, |row| {
                    row.child(div().text_color(palette.text3).child("›"))
                })
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if current { palette.link } else { palette.text3 })
                        .child(format!("{}. {label}", ix + 1)),
                )
        }))
}

/// A row of footer buttons, right-aligned, with optional text on the left.
pub fn footer_row(note: Option<String>, buttons: Vec<AnyElement>, palette: &Palette) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(11.))
                .text_color(palette.text2)
                .children(note),
        )
        .children(buttons)
}

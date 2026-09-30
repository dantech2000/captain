use gpui_kit::component::WindowExt;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::assistant::MigrationAssistant;
use crate::theme::Palette;
use crate::widgets::danger_footer;

/// An item that switches over and the containers it stops in the old engine.
type Stop = (String, Vec<String>);

impl MigrationAssistant {
    /// Starts the run. When items switch over, it first asks, and lists exactly
    /// which containers it stops in the old engine.
    pub(super) fn confirm_start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(plan) = &self.plan else {
            return;
        };
        let stops: Vec<Stop> = plan
            .switch_overs()
            .iter()
            .map(|item| (item.label(), item.running_containers()))
            .collect();
        if stops.is_empty() {
            self.start(cx);
            return;
        }
        let source = plan.source.clone();
        let view = cx.entity().downgrade();
        window.open_alert_dialog(cx, move |alert, _, cx| {
            let view = view.clone();
            alert
                .title("Stop these in the old engine?")
                .description(description(&stops, &source, &Palette::of(cx)))
                .footer(danger_footer(
                    "Stop and switch over",
                    "Stop the other engine and start using Captain Engine.",
                ))
                .on_ok(move |_, _, cx| {
                    view.update(cx, |view, cx| view.start(cx)).ok();
                    true
                })
        });
    }
}

fn description(stops: &[Stop], source: &str, palette: &Palette) -> Div {
    let rows = stops.iter().map(|(item, containers)| {
        // A standalone container is its own only container: name it once.
        let only_itself = containers.len() == 1 && containers[0] == *item;
        div()
            .flex()
            .gap(px(6.))
            .child(div().font_weight(FontWeight::MEDIUM).child(item.clone()))
            .when(!only_itself, |row| {
                row.child(
                    div()
                        .font_family(palette.mono())
                        .text_color(palette.text2)
                        .child(containers.join(", ")),
                )
            })
    });
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .text_size(px(12.))
        .child(format!(
            "Captain stops these containers in {source}. It does not delete them:"
        ))
        .child(div().flex().flex_col().gap(px(4.)).children(rows))
        .child(
            "Then it copies their data again and starts them here. Each item is down \
             for a few seconds. You can roll back from the summary.",
        )
        .child(div().text_color(palette.text2).child(
            "Before it stops anything, Captain checks each item. It refuses a \
                 container started with --rm, because the old engine deletes it when it \
                 stops. It refuses when another running container writes to the item's \
                 volumes, and when a volume or container of the same name is already \
                 here and Captain did not copy it.",
        ))
}

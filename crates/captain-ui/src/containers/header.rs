use captain_core::store::ContainerStore;
use gpui_kit::component::label::Label;
use gpui_kit::component::{ActiveTheme, h_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

/// `store` is `None` until the first list arrives, so the counts stay hidden.
pub fn render(store: Option<&ContainerStore>, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    h_flex()
        .justify_between()
        .items_center()
        .px_4()
        .py_3()
        .border_b_1()
        .border_color(theme.border)
        .child(
            Label::new("Containers")
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD),
        )
        .when_some(store, |this, store| {
            this.child(
                Label::new(format!(
                    "{} running, {} total",
                    store.active_count(),
                    store.len()
                ))
                .text_sm()
                .text_color(theme.muted_foreground),
            )
        })
}

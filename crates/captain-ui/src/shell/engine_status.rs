use captain_core::EngineError;
use captain_core::model::EngineInfo;
use gpui_kit::component::label::Label;
use gpui_kit::component::{ActiveTheme, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

/// The connection to the engine, as the sidebar footer shows it.
#[derive(Debug, Clone)]
pub enum EngineStatus {
    Connecting,
    Connected(EngineInfo),
    Failed(EngineError),
}

pub fn render(status: &EngineStatus, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let (color, text) = match status {
        EngineStatus::Connecting => (theme.warning, "Connecting...".to_string()),
        EngineStatus::Connected(info) => (theme.success, format!("Docker {}", info.version)),
        EngineStatus::Failed(_) => (theme.danger, "Engine not reachable".to_string()),
    };
    let detail = match status {
        EngineStatus::Connecting => String::new(),
        EngineStatus::Connected(info) => info.endpoint.clone(),
        EngineStatus::Failed(error) => error.to_string(),
    };

    v_flex()
        .gap_1()
        .min_w_0()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().size_2().rounded_full().bg(color))
                .child(Label::new(text).text_xs()),
        )
        .when(!detail.is_empty(), |this| {
            this.child(
                Label::new(detail)
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .truncate(),
            )
        })
}

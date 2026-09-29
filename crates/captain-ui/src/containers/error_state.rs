use captain_core::EngineError;
use gpui_kit::assets::IconName;
use gpui_kit::component::label::Label;
use gpui_kit::component::{ActiveTheme, Icon, v_flex};
use gpui_kit::*;

pub fn render(error: &EngineError, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_2()
        .p_8()
        .child(
            Icon::new(IconName::CircleAlert)
                .size_8()
                .text_color(theme.danger),
        )
        .child(Label::new("Docker is not reachable").font_weight(FontWeight::SEMIBOLD))
        .child(
            Label::new(error.to_string())
                .text_sm()
                .text_color(theme.muted_foreground),
        )
        .child(
            Label::new("Start Docker, or set DOCKER_HOST, then restart Captain.")
                .text_sm()
                .text_color(theme.muted_foreground),
        )
}

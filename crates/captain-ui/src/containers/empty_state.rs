use gpui_kit::assets::IconName;
use gpui_kit::component::label::Label;
use gpui_kit::component::{ActiveTheme, Icon, v_flex};
use gpui_kit::*;

pub fn render(cx: &App) -> impl IntoElement {
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_2()
        .text_color(cx.theme().muted_foreground)
        .child(Icon::new(IconName::Container).size_8())
        .child(Label::new("No containers"))
        .child(Label::new("Run `docker run hello-world` and it will show up here.").text_sm())
}

use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;

struct NavItem {
    label: &'static str,
    icon: IconName,
    count: Option<usize>,
    available: bool,
}

/// The Docker sections. Only Containers exists so far; the rest say "Soon".
pub fn render(container_count: usize, palette: &Palette) -> impl IntoElement {
    let items = [
        NavItem {
            label: "Containers",
            icon: IconName::Container,
            count: Some(container_count),
            available: true,
        },
        NavItem {
            label: "Images",
            icon: IconName::Layers,
            count: None,
            available: false,
        },
        NavItem {
            label: "Volumes",
            icon: IconName::HardDrive,
            count: None,
            available: false,
        },
        NavItem {
            label: "Networks",
            icon: IconName::Network,
            count: None,
            available: false,
        },
    ];

    div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .children(items.into_iter().map(|item| {
            let trailing = match item.count {
                Some(count) => count.to_string(),
                None => "Soon".to_string(),
            };
            div()
                .h(px(32.))
                .px(px(10.))
                .flex()
                .items_center()
                .gap(px(10.))
                .rounded(px(8.))
                .when(item.available, |this| {
                    this.bg(palette.nav_selected)
                        .font_weight(FontWeight::SEMIBOLD)
                })
                .text_color(if item.available {
                    palette.text
                } else {
                    palette.text2
                })
                .child(
                    Icon::new(item.icon)
                        .size(px(16.))
                        .text_color(if item.available {
                            palette.accent
                        } else {
                            palette.text3
                        }),
                )
                .child(div().flex_1().child(item.label))
                .child(
                    div()
                        .text_size(px(11.))
                        .font_weight(FontWeight::NORMAL)
                        .text_color(palette.text3)
                        .child(trailing),
                )
        }))
}

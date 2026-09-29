use gpui_kit::component::ActiveTheme;
use gpui_kit::component::label::Label;
use gpui_kit::component::sidebar::{
    Sidebar, SidebarFooter, SidebarHeader, SidebarMenu, SidebarMenuItem,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::engine_status::{self, EngineStatus};
use super::page::Page;

pub fn render(active: Page, status: &EngineStatus, cx: &App) -> impl IntoElement {
    let muted = cx.theme().muted_foreground;
    let items = Page::ALL.into_iter().map(|page| {
        SidebarMenuItem::new(page.label())
            .icon(page.icon())
            .active(page == active)
            .disable(!page.is_available())
            .when(!page.is_available(), |item| {
                item.suffix(move |_, _| Label::new("Soon").text_xs().text_color(muted))
            })
    });

    Sidebar::new("sidebar")
        .w(px(220.))
        .header(SidebarHeader::new().child(Label::new("Captain").font_weight(FontWeight::SEMIBOLD)))
        .child(SidebarMenu::new().children(items))
        .footer(SidebarFooter::new().child(engine_status::render(status, cx)))
}

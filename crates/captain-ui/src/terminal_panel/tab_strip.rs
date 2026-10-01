//! The row above the shells: a tab per shell with its title and a close button,
//! the + button, and the button that hides the panel.

use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::keys::{CLOSE_TAB_KEYS, NEW_TAB_KEYS, TOGGLE_KEYS};
use super::panel_view::TerminalPanel;
use super::tab_title::tab_title;
use crate::help::HelpExt;
use crate::theme::Palette;

pub fn render(panel: &TerminalPanel, palette: &Palette, cx: &mut Context<TerminalPanel>) -> Div {
    let home = std::env::home_dir();
    let tabs = panel.tabs.iter().enumerate().map(|(index, tab)| {
        let title = tab_title(tab.view.read(cx).title(), &tab.cwd, home.as_deref());
        let active = index == panel.active;
        let folder = tab.cwd.display().to_string();
        let close = icon_button(("terminal-tab-close", index), IconName::Close, palette)
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |this, _, _, cx| this.close_tab(index, cx)))
            .help_keys(
                "Close this terminal tab. Its shell and the program it runs end.",
                CLOSE_TAB_KEYS,
            );
        let hover = palette.nav_selected;
        div()
            .id(("terminal-tab", index))
            .h(px(26.))
            .max_w(px(220.))
            .flex()
            .items_center()
            .gap(px(6.))
            .pl(px(10.))
            .pr(px(4.))
            .rounded(px(6.))
            .cursor_pointer()
            .text_size(px(12.))
            .text_color(if active { palette.text } else { palette.text2 })
            .when(active, |tab| tab.bg(palette.nav_selected))
            .hover(move |style| style.bg(hover))
            .on_click(cx.listener(move |this, _, _, cx| this.select(index, cx)))
            .child(Icon::new(IconName::SquareTerminal).size(px(13.)))
            .child(div().min_w_0().truncate().child(title))
            .child(close)
            .help(format!("Show the shell that started in {folder}."))
    });
    let folder = panel.default_dir(cx).display().to_string();
    let add = icon_button("terminal-new-tab", IconName::Plus, palette)
        .on_click(cx.listener(|this, _, _, cx| {
            let dir = this.default_dir(cx);
            this.new_tab(dir, cx);
            this.select(this.tabs.len() - 1, cx);
        }))
        .help_keys(
            format!("Open a new terminal tab in {folder}."),
            NEW_TAB_KEYS,
        );
    let hide = icon_button("terminal-hide", IconName::ChevronDown, palette)
        .on_click(cx.listener(|this, _, _, cx| this.hide(cx)))
        .help_keys(
            "Hide the terminal panel. Its tabs keep running.",
            TOGGLE_KEYS,
        );

    div()
        .flex_shrink_0()
        .h(px(36.))
        .flex()
        .items_center()
        .gap(px(4.))
        .px(px(10.))
        .child(
            div()
                .id("terminal-tabs")
                .flex_1()
                .min_w_0()
                .flex()
                .items_center()
                .gap(px(4.))
                .overflow_x_scroll()
                .children(tabs)
                .child(add),
        )
        .child(hide)
}

fn icon_button(id: impl Into<ElementId>, icon: IconName, palette: &Palette) -> Stateful<Div> {
    let hover = palette.nav_selected;
    div()
        .id(id)
        .flex_shrink_0()
        .size(px(22.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(5.))
        .cursor_pointer()
        .text_color(palette.text2)
        .hover(move |style| style.bg(hover))
        .child(Icon::new(icon).size(px(13.)))
}

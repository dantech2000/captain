use captain_core::model::{breadcrumbs, parent_path};
use gpui_kit::assets::IconName;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::files_pane::FilesPane;
use crate::theme::Palette;
use crate::widgets::icon_button;

/// Up, the breadcrumbs of the current folder, Refresh, and Save to Downloads.
pub fn render(pane: &FilesPane, palette: &Palette, cx: &mut Context<FilesPane>) -> Div {
    let running = pane.running();
    let can_up = running && (pane.preview.is_some() || parent_path(&pane.path).is_some());
    let can_save = running && !pane.saving && pane.save_target().is_some();
    let crumbs = breadcrumbs(&pane.path);
    let last = crumbs.len() - 1;

    div()
        .h(px(26.))
        .flex()
        .items_center()
        .gap(px(6.))
        .child(
            tool(
                "files-up",
                IconName::ArrowUp,
                can_up,
                palette,
                cx.listener(|this, _, _, cx| this.go_up(cx)),
            )
            .tooltip(|window, cx| Tooltip::new("Parent folder").build(window, cx)),
        )
        .child(
            div()
                .id("files-crumbs")
                .flex_1()
                .min_w_0()
                .h_full()
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(7.))
                .bg(palette.field)
                .overflow_x_scroll()
                .font_family(palette.mono())
                .text_size(px(11.))
                .children(crumbs.into_iter().enumerate().map(|(ix, crumb)| {
                    let separator = ix > 1;
                    let target = crumb.path.clone();
                    div()
                        .flex()
                        .flex_shrink_0()
                        .when(separator, |this| {
                            this.child(div().px(px(3.)).text_color(palette.text3).child("/"))
                        })
                        .child(
                            div()
                                .id(("files-crumb", ix))
                                .text_color(if ix == last {
                                    palette.text
                                } else {
                                    palette.accent
                                })
                                .when(ix != last && running, |this| {
                                    this.cursor_pointer().on_click(cx.listener(
                                        move |this, _, _, cx| this.navigate(target.clone(), cx),
                                    ))
                                })
                                .child(crumb.label),
                        )
                })),
        )
        .child(
            tool(
                "files-refresh",
                IconName::RotateCw,
                running,
                palette,
                cx.listener(|this, _, _, cx| {
                    this.preview = None;
                    this.load(None, cx);
                }),
            )
            .tooltip(|window, cx| Tooltip::new("Refresh").build(window, cx)),
        )
        .child(
            tool(
                "files-save",
                IconName::Download,
                can_save,
                palette,
                cx.listener(|this, _, window, cx| this.save(window, cx)),
            )
            .tooltip(|window, cx| Tooltip::new("Save to Downloads").build(window, cx)),
        )
}

/// An icon button that is dimmed and ignores clicks when it does not apply.
fn tool(
    id: &'static str,
    icon: IconName,
    enabled: bool,
    palette: &Palette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    icon_button(id, icon, palette, move |event, window, cx| {
        if enabled {
            on_click(event, window, cx);
        }
    })
    .when(!enabled, |button| button.opacity(0.4).cursor_default())
}

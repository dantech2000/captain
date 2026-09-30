use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::map_state::Zoom;
use crate::help::HelpExt;
use crate::project::ProjectView;
use crate::theme::Palette;

/// What each kind of line means, in the top right of the map.
pub fn legend(palette: &Palette) -> Stateful<Div> {
    let row = |swatch: Div, text: &'static str| {
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(swatch.w(px(22.)).flex_shrink_0())
            .child(text)
    };
    panel(palette)
        .id("map-legend")
        .top(px(16.))
        .flex_col()
        .gap(px(8.))
        .p(px(12.))
        .text_size(px(11.))
        .text_color(palette.text2)
        .child(row(div().h(px(2.)).bg(palette.accent), "Published port"))
        .child(row(div().h(px(2.)).bg(palette.text2), "Talks to (from env)"))
        .child(row(
            div().border_t_2().border_dashed().border_color(palette.info),
            "Mounts",
        ))
        .help("Solid colored lines reach a service from a port on this Mac. Gray lines join services whose environment names another. Dashed lines lead to volumes.")
}

/// Fit and 100%, in the bottom right of the map.
pub fn zoom(zoom: Zoom, scale: f32, view: &WeakEntity<ProjectView>, palette: &Palette) -> Div {
    let choice = |id: &'static str, label: String, value: Zoom, help: &'static str| {
        let view = view.clone();
        let selected = zoom == value;
        let hover = palette.nav_selected;
        div()
            .id(id)
            .h(px(28.))
            .px(px(10.))
            .flex()
            .items_center()
            .rounded(px(7.))
            .text_size(px(11.))
            .text_color(if selected {
                palette.text
            } else {
                palette.text2
            })
            .when(selected, |this| this.bg(palette.segment))
            .cursor_pointer()
            .hover(move |style| style.bg(hover))
            .on_click(move |_, _, cx| {
                view.update(cx, |view, cx| {
                    view.map.zoom = value;
                    cx.notify();
                })
                .ok();
            })
            .child(label)
            .help(help)
    };
    let fit = format!("Fit · {:.0}%", scale * 100.);
    panel(palette)
        .bottom(px(16.))
        .gap(px(4.))
        .p(px(4.))
        .child(choice(
            "map-zoom-fit",
            fit,
            Zoom::Fit,
            "Shrink the map to the width of the page.",
        ))
        .child(choice(
            "map-zoom-actual",
            "100%".into(),
            Zoom::Actual,
            "Show the map at full size. Scroll to see the rest.",
        ))
}

fn panel(palette: &Palette) -> Div {
    div()
        .absolute()
        .right(px(16.))
        .flex()
        .rounded(px(12.))
        .bg(palette.card.alpha(0.94))
        .border_1()
        .border_color(palette.sep)
}

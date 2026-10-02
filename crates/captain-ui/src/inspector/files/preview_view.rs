use captain_core::format::bytes_label;
use captain_core::model::PREVIEW_LIMIT;
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use super::files_pane::{FilesPane, Load, Preview, note};
use crate::theme::Palette;
use crate::widgets::skeleton_lines;

const LINE_HEIGHT: f32 = 16.;

/// The read-only preview: a Back row with the file name and size, then the text.
pub fn render(preview: &Preview, palette: &Palette, cx: &mut Context<FilesPane>) -> Div {
    let size = match &preview.load {
        Load::Loaded((file, _)) => bytes_label(file.size),
        _ => String::new(),
    };
    let header = div()
        .h(px(24.))
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .id("files-preview-back")
                .flex()
                .items_center()
                .gap(px(4.))
                .text_size(px(12.))
                .text_color(palette.link)
                .cursor_pointer()
                .on_click(cx.listener(|this, _, _, cx| this.close_preview(cx)))
                .child(Icon::new(IconName::ArrowLeft).size(px(12.)))
                .child("Back"),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .font_weight(FontWeight::SEMIBOLD)
                .child(preview.name.clone()),
        )
        .child(
            div()
                .flex_shrink_0()
                .text_size(px(11.))
                .text_color(palette.text3)
                .child(size),
        );

    let body = match &preview.load {
        Load::Idle | Load::Loading => div().flex_1().child(skeleton_lines(8).p(px(12.))),
        Load::Failed(error) => note(
            IconName::CircleAlert,
            "Could not read this file",
            error.clone(),
            palette,
        ),
        Load::Loaded((file, _)) if file.text().is_none() => note(
            IconName::File,
            "Binary file, no preview",
            "Save it to Downloads to open it.".into(),
            palette,
        ),
        Load::Loaded((file, lines)) => {
            let lines = lines.clone();
            let text = palette.text;
            // Long lines scroll sideways. The list is as wide as its longest line.
            let longest = (0..lines.len()).max_by_key(|&ix| lines[ix].len());
            let list = uniform_list("file-preview", lines.len(), move |range, _, _| {
                range
                    .map(|ix| {
                        div()
                            .h(px(LINE_HEIGHT))
                            .whitespace_nowrap()
                            .text_color(text)
                            .child(lines[ix].clone())
                    })
                    .collect()
            })
            .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
            .with_width_from_item(longest)
            .flex_1();
            let truncated = file.is_truncated().then(|| {
                div()
                    .px(px(10.))
                    .py(px(6.))
                    .border_t_1()
                    .border_color(palette.sep)
                    .text_size(px(11.))
                    .text_color(palette.text2)
                    .child(format!(
                        "Showing the first {}. Save the file to see all of it.",
                        bytes_label(PREVIEW_LIMIT)
                    ))
            });
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .rounded(px(9.))
                .border_1()
                .border_color(palette.sep)
                .bg(palette.card)
                .overflow_hidden()
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .px(px(10.))
                        .py(px(6.))
                        .font_family(palette.mono())
                        .text_size(px(11.))
                        .child(list),
                )
                .children(truncated)
        }
    };

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(header)
        .child(body)
}

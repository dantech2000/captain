use captain_core::format::bytes_label;
use captain_core::store::ImageFilter;
use gpui_kit::*;

use super::ImagesState;
use crate::theme::Palette;
use crate::widgets::{Segment, page_header, segmented};

/// The page title, the image count and total size, and the filter.
pub fn render(
    handle: &Entity<ImagesState>,
    state: &ImagesState,
    palette: &Palette,
) -> impl IntoElement {
    let store = state.store();
    let summary = if !state.is_loaded() {
        "Loading...".to_string()
    } else {
        let count = match store.len() {
            1 => "1 image".to_string(),
            n => format!("{n} images"),
        };
        let mut summary = format!("{count} · {}", bytes_label(store.total_size()));
        let dangling = store.count(ImageFilter::Dangling);
        if dangling > 0 {
            summary.push_str(&format!(" · {dangling} dangling"));
        }
        summary
    };

    let segments = ImageFilter::ALL
        .into_iter()
        .map(|filter| {
            let handle = handle.clone();
            Segment {
                label: filter.label().into(),
                selected: state.filter() == filter,
                on_click: Box::new(move |_, cx| {
                    handle.update(cx, |state, cx| state.set_filter(filter, cx));
                }),
            }
        })
        .collect();
    let filter = segmented("image-filter", segments, palette)
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());

    page_header(
        "images-header",
        "Images",
        summary,
        Some(filter.into_any_element()),
        palette,
    )
}

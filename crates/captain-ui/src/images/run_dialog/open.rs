use captain_core::model::ImageDetail;
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::RunDialog;
use crate::images::ImagesState;

/// Opens the Run dialog for `image`, a tag or an ID, prefilled from `detail`.
pub fn open(
    state: Entity<ImagesState>,
    image: String,
    detail: &ImageDetail,
    window: &mut Window,
    cx: &mut App,
) {
    let title = SharedString::from(format!("Run {image}"));
    let form = cx.new(|cx| RunDialog::new(state, image, detail, window, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title(title.clone())
            .w(px(520.))
            .overlay_closable(false)
            .child(form.clone())
    });
}

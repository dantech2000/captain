use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::BuildDialog;
use crate::images::ImagesState;

/// Opens the Build dialog. Closing it stops a running build.
pub fn open(state: Entity<ImagesState>, window: &mut Window, cx: &mut App) {
    let form = cx.new(|cx| BuildDialog::new(state, window, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("Build image")
            .w(px(640.))
            .overlay_closable(false)
            .child(form.clone())
    });
}

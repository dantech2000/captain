use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::MigrationAssistant;
use crate::workspace::Workspace;

/// Opens the Migration Assistant over the window. It copies into the engine the
/// workspace is connected to.
pub fn open(workspace: Entity<Workspace>, window: &mut Window, cx: &mut App) {
    let assistant = cx.new(|cx| MigrationAssistant::new(&workspace, window, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("Bring data from another engine")
            .w(px(720.))
            .overlay_closable(false)
            .child(assistant.clone())
    });
}

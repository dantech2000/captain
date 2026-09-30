//! Save to Downloads: copies a file, or a folder as a `.tar`, out of the container.

use std::path::Path;

use captain_core::model::{FileKind, join_path};
use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;

use crate::widgets::error_notification;
use gpui_kit::*;

use super::files_pane::FilesPane;

impl FilesPane {
    /// The container path that Save copies: the previewed file, else the selected
    /// file or folder. Links and devices cannot be saved.
    pub(super) fn save_target(&self) -> Option<String> {
        if let Some(preview) = &self.preview {
            return Some(preview.path.clone());
        }
        self.selected_entry()
            .filter(|entry| matches!(entry.kind, FileKind::File | FileKind::Folder))
            .map(|entry| join_path(&self.path, &entry.name))
    }

    /// Saves [`Self::save_target`] to the Downloads folder and reports the result as
    /// a toast.
    pub(super) fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(engine), Some((id, _)), Some(path)) =
            (self.engine.clone(), self.target.clone(), self.save_target())
        else {
            return;
        };
        let Some(dir) = dirs::download_dir().or_else(dirs::home_dir) else {
            let note = error_notification(
                "Could not save the file",
                "Captain could not find your Downloads folder.",
            );
            window.push_notification(note, cx);
            return;
        };
        if self.saving {
            return;
        }
        self.saving = true;
        cx.notify();
        let save = engine.save_path(&id, &path, &dir);
        cx.spawn_in(window, async move |this, cx| {
            let result = save.await;
            this.update_in(cx, |this, window, cx| {
                this.saving = false;
                let note = match result {
                    Ok(saved) => Notification::success(format!("Saved to {}", short(&saved))),
                    Err(error) => {
                        error_notification(format!("Could not save {path}"), error.to_string())
                    }
                };
                window.push_notification(note, cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

/// `path` with the home folder as `~`.
fn short(path: &Path) -> String {
    match dirs::home_dir().and_then(|home| path.strip_prefix(home).ok().map(Path::to_path_buf)) {
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}

//! Moving around the container's folders, and opening the preview.

use std::rc::Rc;

use captain_core::model::{FileKind, PREVIEW_LIMIT, join_path, parent_path, sort_entries};
use gpui_kit::*;

use super::files_pane::{FilesPane, Load, Preview};

impl FilesPane {
    /// Lists the current folder. `select` names the entry to select afterwards, for
    /// example the folder the user just left.
    pub(super) fn load(&mut self, select: Option<String>, cx: &mut Context<Self>) {
        let (Some(engine), Some((id, true))) = (self.engine.clone(), self.target.clone()) else {
            return;
        };
        self.listing = Load::Loading;
        self.selected = None;
        let list = engine.list_files(&id, &self.path);
        self.load_task = Some(cx.spawn(async move |this, cx| {
            let result = list.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok(mut entries) => {
                        sort_entries(&mut entries);
                        this.selected =
                            select.and_then(|name| entries.iter().position(|e| e.name == name));
                        if let Some(ix) = this.selected {
                            this.scroll.scroll_to_item(ix, ScrollStrategy::Center);
                        }
                        this.listing = Load::Loaded(Rc::new(entries));
                    }
                    Err(error) => this.listing = Load::Failed(error.to_string()),
                }
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    /// Opens the folder `path`.
    pub(super) fn navigate(&mut self, path: String, cx: &mut Context<Self>) {
        self.path = path;
        self.preview = None;
        self.preview_task = None;
        self.load(None, cx);
    }

    /// Goes to the parent folder and selects the folder it came from. With a preview
    /// open, closes the preview instead.
    pub(super) fn go_up(&mut self, cx: &mut Context<Self>) {
        if self.preview.is_some() {
            self.close_preview(cx);
            return;
        }
        let Some(parent) = parent_path(&self.path) else {
            return;
        };
        let child = self.path.rsplit('/').next().map(String::from);
        self.path = parent;
        self.load(child, cx);
    }

    pub(super) fn close_preview(&mut self, cx: &mut Context<Self>) {
        self.preview = None;
        self.preview_task = None;
        cx.notify();
    }

    pub(super) fn select(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.selected = Some(ix);
        cx.notify();
    }

    /// Moves the selection up (`-1`) or down (`1`), and keeps it in view.
    pub(super) fn move_selection(&mut self, step: isize, cx: &mut Context<Self>) {
        let Some(count) = self.entries().map(|e| e.len()).filter(|n| *n > 0) else {
            return;
        };
        let ix = match self.selected {
            Some(ix) => ix.saturating_add_signed(step).min(count - 1),
            None if step < 0 => count - 1,
            None => 0,
        };
        self.selected = Some(ix);
        self.scroll.scroll_to_item(ix, ScrollStrategy::Nearest);
        cx.notify();
    }

    /// Opens the selected folder, or previews the selected file.
    pub(super) fn open_selected(&mut self, cx: &mut Context<Self>) {
        if self.preview.is_some() {
            return;
        }
        let Some(entry) = self.selected_entry().cloned() else {
            return;
        };
        let path = join_path(&self.path, &entry.name);
        if entry.opens {
            self.navigate(path, cx);
        } else if entry.kind == FileKind::File {
            self.open_preview(path, entry.name, cx);
        }
    }

    fn open_preview(&mut self, path: String, name: String, cx: &mut Context<Self>) {
        let (Some(engine), Some((id, _))) = (self.engine.clone(), self.target.clone()) else {
            return;
        };
        let read = engine.read_file(&id, &path, PREVIEW_LIMIT);
        self.preview = Some(Preview {
            path,
            name,
            load: Load::Loading,
        });
        self.preview_task = Some(cx.spawn(async move |this, cx| {
            let result = read.await;
            this.update(cx, |this, cx| {
                if let Some(preview) = &mut this.preview {
                    preview.load = match result {
                        Ok(file) => {
                            let lines = file
                                .text()
                                .map(|text| text.lines().map(|l| l.to_string().into()).collect())
                                .unwrap_or_default();
                            Load::Loaded((file, Rc::new(lines)))
                        }
                        Err(error) => Load::Failed(error.to_string()),
                    };
                }
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }
}

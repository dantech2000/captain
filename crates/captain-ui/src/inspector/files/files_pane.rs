use std::rc::Rc;
use std::sync::Arc;

use captain_core::Engine;
use captain_core::model::{FileEntry, FilePreview};
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use super::keys::{self, ClosePreview, GoUp, KEY_CONTEXT, OpenSelected, SelectNext, SelectPrev};
use super::{file_list, path_bar, preview_view};
use crate::theme::Palette;

/// Where a load is.
pub(super) enum Load<T> {
    /// Nothing loaded yet. The next show of the tab loads.
    Idle,
    Loading,
    Loaded(T),
    Failed(String),
}

/// The file the preview shows.
pub(super) struct Preview {
    pub path: String,
    pub name: String,
    pub load: Load<(FilePreview, Rc<Vec<SharedString>>)>,
}

/// The Files tab: one folder of the selected container at a time, and a read-only
/// preview of a file. It loads only while the tab shows.
pub struct FilesPane {
    pub(super) engine: Option<Arc<dyn Engine>>,
    /// The container's ID, and whether it runs.
    pub(super) target: Option<(String, bool)>,
    pub(super) path: String,
    pub(super) listing: Load<Rc<Vec<FileEntry>>>,
    pub(super) selected: Option<usize>,
    pub(super) preview: Option<Preview>,
    pub(super) saving: bool,
    pub(super) focus: FocusHandle,
    pub(super) scroll: UniformListScrollHandle,
    pub(super) load_task: Option<Task<()>>,
    pub(super) preview_task: Option<Task<()>>,
}

impl FilesPane {
    pub fn new(cx: &mut Context<Self>) -> Self {
        keys::bind(cx);
        Self {
            engine: None,
            target: None,
            path: "/".into(),
            listing: Load::Idle,
            selected: None,
            preview: None,
            saving: false,
            focus: cx.focus_handle(),
            scroll: UniformListScrollHandle::new(),
            load_task: None,
            preview_task: None,
        }
    }

    /// Follows the inspector's container. Another container starts again at `/`; a
    /// stop drops the listing.
    pub fn set_target(
        &mut self,
        target: Option<(String, bool)>,
        engine: Option<Arc<dyn Engine>>,
        cx: &mut Context<Self>,
    ) {
        let same = target.as_ref().map(|t| &t.0) == self.target.as_ref().map(|t| &t.0);
        if !same {
            self.path = "/".into();
        }
        if !same || target.as_ref().is_some_and(|t| !t.1) {
            self.reset();
        }
        self.target = target;
        self.engine = engine;
        cx.notify();
    }

    /// The tab is showing: load the folder if nothing is loaded.
    pub fn show(&mut self, cx: &mut Context<Self>) {
        if matches!(self.listing, Load::Idle) {
            self.load(None, cx);
        }
    }

    pub(super) fn reset(&mut self) {
        self.listing = Load::Idle;
        self.selected = None;
        self.preview = None;
        self.load_task = None;
        self.preview_task = None;
    }

    pub(super) fn running(&self) -> bool {
        self.target.as_ref().is_some_and(|t| t.1)
    }

    pub(super) fn entries(&self) -> Option<&Rc<Vec<FileEntry>>> {
        match &self.listing {
            Load::Loaded(entries) => Some(entries),
            _ => None,
        }
    }

    pub(super) fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries()?.get(self.selected?)
    }
}

impl Render for FilesPane {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let body = if !self.running() {
            note(
                IconName::Folder,
                "Container is not running",
                "Start the container to browse its files.".into(),
                &palette,
            )
            .into_any_element()
        } else if let Some(preview) = &self.preview {
            preview_view::render(preview, &palette, cx).into_any_element()
        } else {
            match &self.listing {
                Load::Idle | Load::Loading => loading(&palette).into_any_element(),
                Load::Failed(error) => note(
                    IconName::CircleAlert,
                    "Could not list this folder",
                    error.clone(),
                    &palette,
                )
                .into_any_element(),
                Load::Loaded(entries) if entries.is_empty() => note(
                    IconName::FolderOpen,
                    "This folder is empty",
                    String::new(),
                    &palette,
                )
                .into_any_element(),
                Load::Loaded(entries) => file_list::render(
                    entries.clone(),
                    self.selected,
                    &self.scroll,
                    &palette,
                    cx.entity().downgrade(),
                )
                .into_any_element(),
            }
        };

        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .gap(px(10.))
            .px(px(20.))
            .pt(px(14.))
            .pb(px(20.))
            .track_focus(&self.focus)
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(|this, _: &SelectPrev, _, cx| this.move_selection(-1, cx)))
            .on_action(cx.listener(|this, _: &SelectNext, _, cx| this.move_selection(1, cx)))
            .on_action(cx.listener(|this, _: &OpenSelected, _, cx| this.open_selected(cx)))
            .on_action(cx.listener(|this, _: &GoUp, _, cx| this.go_up(cx)))
            .on_action(cx.listener(|this, _: &ClosePreview, _, cx| this.close_preview(cx)))
            .child(path_bar::render(self, &palette, cx))
            .child(body)
    }
}

fn loading(palette: &Palette) -> Div {
    div()
        .flex_1()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(12.))
        .text_color(palette.text2)
        .child("Loading…")
}

/// A centered icon, title, and detail, for states with no list.
pub(super) fn note(icon: IconName, title: &'static str, detail: String, palette: &Palette) -> Div {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(8.))
        .p(px(24.))
        .child(Icon::new(icon).size(px(28.)).text_color(palette.text3))
        .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
        .child(
            div()
                .text_size(px(12.))
                .text_color(palette.text2)
                .text_center()
                .child(detail),
        )
}

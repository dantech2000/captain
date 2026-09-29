use std::collections::HashSet;
use std::sync::Arc;

use captain_core::Engine;
use captain_core::model::{Image, ImageDetail, ImageLayer};
use captain_core::store::{ImageFilter, ImageStore, PullTracker};
use gpui_kit::*;

use super::Started;

/// The Images page state: the image list, the filter, the selection and its details,
/// and any running remove, prune, or pull. The view hands it the engine once the
/// workspace connects.
pub struct ImagesState {
    pub(super) engine: Option<Arc<dyn Engine>>,
    pub(super) store: ImageStore,
    pub(super) filter: ImageFilter,
    pub(super) selected: Option<String>,
    pub(super) loaded: bool,
    /// The last list error.
    pub(super) load_error: Option<String>,
    /// The last remove or prune error.
    pub(super) error: Option<String>,
    /// The result of the last prune, for example "Reclaimed 120 MB".
    pub(super) notice: Option<String>,
    pub(super) removing: HashSet<String>,
    pub(super) pruning: bool,
    pub(super) pull: Option<PullTracker>,
    pub(super) pull_error: Option<String>,
    pub(super) reload_task: Option<Task<()>>,
    pub(super) pull_task: Option<Task<()>>,
    /// The inspect result of the selected image, once it arrives.
    pub(super) detail: Option<ImageDetail>,
    /// The history of the selected image, once it arrives.
    pub(super) layers: Option<Vec<ImageLayer>>,
    pub(super) detail_error: Option<String>,
    pub(super) detail_task: Option<Task<()>>,
    /// The container that the last run started.
    pub(super) started: Option<Started>,
}

impl ImagesState {
    pub fn new() -> Self {
        Self {
            engine: None,
            store: ImageStore::default(),
            filter: ImageFilter::default(),
            selected: None,
            loaded: false,
            load_error: None,
            error: None,
            notice: None,
            removing: HashSet::new(),
            pruning: false,
            pull: None,
            pull_error: None,
            reload_task: None,
            pull_task: None,
            detail: None,
            layers: None,
            detail_error: None,
            detail_task: None,
            started: None,
        }
    }

    /// Starts loading images. A second call with the same engine does nothing.
    pub fn attach(&mut self, engine: Arc<dyn Engine>, cx: &mut Context<Self>) {
        if self
            .engine
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &engine))
        {
            return;
        }
        self.engine = Some(engine);
        self.reload(std::time::Duration::ZERO, cx);
    }

    /// True once the first image list has arrived.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn store(&self) -> &ImageStore {
        &self.store
    }

    pub fn filter(&self) -> ImageFilter {
        self.filter
    }

    pub fn set_filter(&mut self, filter: ImageFilter, cx: &mut Context<Self>) {
        self.filter = filter;
        cx.notify();
    }

    pub fn selected(&self) -> Option<&Image> {
        self.selected.as_deref().and_then(|id| self.store.find(id))
    }

    /// Selects an image and loads its details and history.
    pub fn select(&mut self, id: String, cx: &mut Context<Self>) {
        if self.selected.as_ref() != Some(&id) {
            self.load_detail(&id, cx);
        }
        self.selected = Some(id);
        cx.notify();
    }

    /// Clears the selection, which closes the inspector.
    pub fn deselect(&mut self, cx: &mut Context<Self>) {
        self.selected = None;
        self.clear_detail();
        cx.notify();
    }

    pub fn load_error(&self) -> Option<&str> {
        self.load_error.as_deref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    pub fn is_removing(&self, id: &str) -> bool {
        self.removing.contains(id)
    }

    pub fn is_pruning(&self) -> bool {
        self.pruning
    }

    /// The running or last finished pull.
    pub fn pull(&self) -> Option<&PullTracker> {
        self.pull.as_ref()
    }

    /// True while a pull stream is open.
    pub fn is_pulling(&self) -> bool {
        self.pull_task.is_some()
    }

    pub fn pull_error(&self) -> Option<&str> {
        self.pull_error.as_deref()
    }
}

impl Default for ImagesState {
    fn default() -> Self {
        Self::new()
    }
}

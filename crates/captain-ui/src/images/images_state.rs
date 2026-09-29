use std::collections::HashSet;
use std::sync::Arc;

use captain_core::model::{Image, ImageDetail, ImageLayer};
use captain_core::store::{ImageFilter, ImageStore, PullTracker, PushTracker};
use captain_core::{Engine, ImageBuilder};
use gpui_kit::*;

use super::Started;

/// The Images page state: the image list, the filter, the selection and its details,
/// and any running remove, prune, pull, or push. The view hands it the engine and the
/// builder once the workspace connects.
pub struct ImagesState {
    pub(super) engine: Option<Arc<dyn Engine>>,
    /// Counts engine switches. Remove carries the number it was shown for, and does
    /// nothing after a switch. Results that arrive after a switch are dropped.
    pub(super) generation: u64,
    /// Runs `docker buildx build`. `None` turns the Build button off.
    pub(super) builder: Option<Arc<dyn ImageBuilder>>,
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
    pub(super) push: Option<PushTracker>,
    pub(super) push_error: Option<String>,
    pub(super) push_task: Option<Task<()>>,
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
            generation: 0,
            builder: None,
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
            push: None,
            push_error: None,
            push_task: None,
            detail: None,
            layers: None,
            detail_error: None,
            detail_task: None,
            started: None,
        }
    }

    /// Starts loading images. A second call with the same engine does nothing. A new
    /// engine drops the old engine's list and selection at once.
    pub fn attach(
        &mut self,
        engine: Arc<dyn Engine>,
        builder: Option<Arc<dyn ImageBuilder>>,
        cx: &mut Context<Self>,
    ) {
        self.builder = builder;
        if self
            .engine
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &engine))
        {
            return;
        }
        self.engine = Some(engine);
        self.clear_engine_state(cx);
        self.reload(std::time::Duration::ZERO, cx);
    }

    /// Drops the engine and everything loaded from it, while the workspace has none.
    pub fn detach(&mut self, cx: &mut Context<Self>) {
        self.builder = None;
        if self.engine.take().is_some() {
            self.clear_engine_state(cx);
        }
    }

    /// Drops what came from the old engine. Dropping a task cancels its pull or push.
    fn clear_engine_state(&mut self, cx: &mut Context<Self>) {
        self.generation += 1;
        self.store = ImageStore::default();
        self.loaded = false;
        self.selected = None;
        self.clear_detail();
        self.reload_task = None;
        self.removing.clear();
        self.load_error = None;
        self.error = None;
        self.notice = None;
        self.pruning = false;
        self.pull = None;
        self.pull_error = None;
        self.pull_task = None;
        self.push = None;
        self.push_error = None;
        self.push_task = None;
        self.started = None;
        cx.notify();
    }

    /// The number of the current engine. See [`ImagesState::remove`].
    pub fn generation(&self) -> u64 {
        self.generation
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

    /// Loads the selected image's details again, for example after a new tag.
    pub fn reload_selected_detail(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected.clone() {
            self.load_detail(&id, cx);
        }
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

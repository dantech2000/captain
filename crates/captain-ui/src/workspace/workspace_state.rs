use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use captain_core::extension::ExtensionManager;
use captain_core::model::{Container, EngineInfo, ProjectAction};
use captain_core::store::{
    ContainerFilter, ContainerStore, GroupKey, MultiSelection, SelectMode, StatsBoard,
};
use captain_core::{Engine, EngineError, ImageBuilder, ProjectRunner};
use gpui_kit::*;

use super::{InspectorTab, LogFilter, Page, WorkspaceEvent};

/// The state of the engine connection.
#[derive(Debug, Clone)]
pub enum Connection {
    Connecting,
    Connected(EngineInfo),
    Failed(EngineError),
}

pub struct Workspace {
    pub(super) engine: Option<Arc<dyn Engine>>,
    /// Counts reconnects. A confirmation carries the number it opened with, and does
    /// nothing after a switch to another engine.
    pub(super) generation: u64,
    pub(super) connection: Connection,
    pub(super) page: Page,
    /// Item counts that the Images, Volumes, and Networks pages report for the sidebar.
    pub(super) page_counts: HashMap<Page, usize>,
    pub(super) loaded: bool,
    pub(super) store: ContainerStore,
    pub(super) stats: StatsBoard,
    pub(super) filter: ContainerFilter,
    pub(super) selected: Option<String>,
    /// The rows selected for a bulk action. It holds the selected row too.
    pub(super) checked: MultiSelection,
    pub(super) reload_task: Option<Task<()>>,
    pub(super) events_task: Option<Task<()>>,
    /// The task that follows each running container's stats, and its number.
    pub(super) stats_tasks: HashMap<String, (u64, Task<()>)>,
    /// The number of the last stats task.
    pub(super) stats_generation: u64,
    pub(super) pending: HashSet<String>,
    /// Cards the user folded.
    pub(super) collapsed: HashSet<GroupKey>,
    /// Show Kubernetes pod containers, one card per namespace.
    pub(super) show_kubernetes: bool,
    /// Runs `docker compose`. `None` when the CLI is missing.
    pub(super) projects: Option<Arc<dyn ProjectRunner>>,
    /// Runs `docker buildx build`. `None` when the CLI or the plugin is missing.
    pub(super) builder: Option<Arc<dyn ImageBuilder>>,
    /// Installs extensions and answers their pages. `None` until connected.
    pub(super) extensions: Option<Arc<dyn ExtensionManager>>,
    /// The Compose command running on each project.
    pub(super) project_pending: HashMap<String, ProjectAction>,
    /// The project the Containers page shows alone, if any.
    pub(super) project_filter: Option<String>,
    /// The sidebar entry the Project page shows.
    pub(super) focus: Option<GroupKey>,
    /// True while the Project page shows the inspector next to its cards.
    pub(super) card_open: bool,
    /// The inspector tab a card button asked for, until the inspector takes it.
    pub(super) inspector_tab: Option<InspectorTab>,
    /// The Logs tab filters a ⌘K command asked for, until the inspector takes them.
    pub(super) log_filter: Option<LogFilter>,
}

impl EventEmitter<WorkspaceEvent> for Workspace {}

impl Workspace {
    pub fn new() -> Self {
        Self {
            engine: None,
            generation: 0,
            connection: Connection::Connecting,
            page: Page::default(),
            page_counts: HashMap::new(),
            loaded: false,
            store: ContainerStore::default(),
            stats: StatsBoard::default(),
            filter: ContainerFilter::default(),
            selected: None,
            checked: MultiSelection::default(),
            reload_task: None,
            events_task: None,
            stats_tasks: HashMap::new(),
            stats_generation: 0,
            pending: HashSet::new(),
            collapsed: HashSet::new(),
            show_kubernetes: false,
            projects: None,
            builder: None,
            extensions: None,
            project_pending: HashMap::new(),
            project_filter: None,
            focus: None,
            card_open: false,
            inspector_tab: None,
            log_filter: None,
        }
    }

    pub fn engine(&self) -> Option<Arc<dyn Engine>> {
        self.engine.clone()
    }

    /// The number of the current connection. See [`Workspace::generation`].
    pub fn engine_generation(&self) -> u64 {
        self.generation
    }

    /// The image builder, once connected, if `docker buildx` is installed.
    pub fn image_builder(&self) -> Option<Arc<dyn ImageBuilder>> {
        self.builder.clone()
    }

    /// The extension manager for the connected engine.
    pub fn extension_manager(&self) -> Option<Arc<dyn ExtensionManager>> {
        self.extensions.clone()
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// True once the first container list has arrived.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn store(&self) -> &ContainerStore {
        &self.store
    }

    pub fn stats(&self) -> &StatsBoard {
        &self.stats
    }

    pub fn page(&self) -> Page {
        self.page
    }

    pub fn set_page(&mut self, page: Page, cx: &mut Context<Self>) {
        self.page = page;
        cx.notify();
    }

    /// The item count for a page's sidebar entry, once the page has loaded.
    pub fn page_count(&self, page: Page) -> Option<usize> {
        match page {
            Page::Containers => self
                .loaded
                .then(|| self.store.shown(self.show_kubernetes).count()),
            _ => self.page_counts.get(&page).copied(),
        }
    }

    /// Records a page's item count. Notifies only when it changes.
    pub fn set_page_count(&mut self, page: Page, count: usize, cx: &mut Context<Self>) {
        if self.page_counts.insert(page, count) != Some(count) {
            cx.notify();
        }
    }

    pub fn filter(&self) -> ContainerFilter {
        self.filter
    }

    pub fn set_filter(&mut self, filter: ContainerFilter, cx: &mut Context<Self>) {
        self.filter = filter;
        cx.notify();
    }

    pub fn selected(&self) -> Option<&Container> {
        self.selected.as_deref().and_then(|id| self.store.find(id))
    }

    pub fn select(&mut self, id: String, cx: &mut Context<Self>) {
        self.checked.click(&id, SelectMode::Replace, &[]);
        self.selected = Some(id);
        cx.notify();
    }

    /// True if the card `key` shows only its header.
    pub fn is_collapsed(&self, key: &GroupKey) -> bool {
        self.collapsed.contains(key)
    }

    /// Folds or unfolds the card `key`.
    pub fn toggle_collapsed(&mut self, key: GroupKey, cx: &mut Context<Self>) {
        if !self.collapsed.remove(&key) {
            self.collapsed.insert(key);
        }
        cx.notify();
    }

    /// True if the list shows Kubernetes pod containers.
    pub fn show_kubernetes(&self) -> bool {
        self.show_kubernetes
    }

    /// The active and total counts of the containers the list shows.
    pub fn shown_counts(&self) -> (usize, usize) {
        let shown = self.store.shown(self.show_kubernetes);
        shown.fold((0, 0), |(active, total), c| {
            (active + usize::from(c.state.is_active()), total + 1)
        })
    }

    pub fn set_show_kubernetes(&mut self, show: bool, cx: &mut Context<Self>) {
        self.show_kubernetes = show;
        cx.notify();
    }

    pub(super) fn fail(&mut self, error: EngineError, cx: &mut Context<Self>) {
        tracing::warn!(%error, "engine connection failed");
        self.connection = Connection::Failed(error);
        self.events_task = None;
        self.reload_task = None;
        self.stats_tasks.clear();
        cx.notify();
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

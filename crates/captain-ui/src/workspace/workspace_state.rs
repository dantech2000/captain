use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use captain_core::model::{Container, EngineInfo, ProjectAction};
use captain_core::store::{ContainerFilter, ContainerStore, StatsBoard};
use captain_core::{Engine, EngineError, ProjectRunner};
use gpui_kit::*;

use super::{Page, WorkspaceEvent};

/// The state of the engine connection.
#[derive(Debug, Clone)]
pub enum Connection {
    Connecting,
    Connected(EngineInfo),
    Failed(EngineError),
}

pub struct Workspace {
    pub(super) engine: Option<Arc<dyn Engine>>,
    pub(super) connection: Connection,
    pub(super) page: Page,
    /// Item counts that the Images, Volumes, and Networks pages report for the sidebar.
    pub(super) page_counts: HashMap<Page, usize>,
    pub(super) loaded: bool,
    pub(super) store: ContainerStore,
    pub(super) stats: StatsBoard,
    pub(super) filter: ContainerFilter,
    pub(super) selected: Option<String>,
    pub(super) reload_task: Option<Task<()>>,
    pub(super) events_task: Option<Task<()>>,
    pub(super) stats_tasks: HashMap<String, Task<()>>,
    pub(super) pending: HashSet<String>,
    /// Project cards the user folded. `None` is the standalone card.
    pub(super) collapsed: HashSet<Option<String>>,
    /// Runs `docker compose`. `None` when the CLI is missing.
    pub(super) projects: Option<Arc<dyn ProjectRunner>>,
    /// The Compose command running on each project.
    pub(super) project_pending: HashMap<String, ProjectAction>,
    /// The project the Containers page shows alone, if any.
    pub(super) project_filter: Option<String>,
}

impl EventEmitter<WorkspaceEvent> for Workspace {}

impl Workspace {
    pub fn new() -> Self {
        Self {
            engine: None,
            connection: Connection::Connecting,
            page: Page::default(),
            page_counts: HashMap::new(),
            loaded: false,
            store: ContainerStore::default(),
            stats: StatsBoard::default(),
            filter: ContainerFilter::default(),
            selected: None,
            reload_task: None,
            events_task: None,
            stats_tasks: HashMap::new(),
            pending: HashSet::new(),
            collapsed: HashSet::new(),
            projects: None,
            project_pending: HashMap::new(),
            project_filter: None,
        }
    }

    pub fn engine(&self) -> Option<Arc<dyn Engine>> {
        self.engine.clone()
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
            Page::Containers => self.loaded.then(|| self.store.len()),
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
        self.selected = Some(id);
        cx.notify();
    }

    /// True if the card for `project` shows only its header.
    pub fn is_collapsed(&self, project: &Option<String>) -> bool {
        self.collapsed.contains(project)
    }

    /// Folds or unfolds the card for `project`.
    pub fn toggle_collapsed(&mut self, project: Option<String>, cx: &mut Context<Self>) {
        if !self.collapsed.remove(&project) {
            self.collapsed.insert(project);
        }
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

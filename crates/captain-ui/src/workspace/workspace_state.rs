use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use captain_core::model::{Container, EngineInfo};
use captain_core::store::{ContainerFilter, ContainerStore, StatsBoard};
use captain_core::{Engine, EngineError};
use gpui_kit::*;

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
    pub(super) loaded: bool,
    pub(super) store: ContainerStore,
    pub(super) stats: StatsBoard,
    pub(super) filter: ContainerFilter,
    pub(super) selected: Option<String>,
    pub(super) reload_task: Option<Task<()>>,
    pub(super) events_task: Option<Task<()>>,
    pub(super) stats_tasks: HashMap<String, Task<()>>,
    pub(super) pending: HashSet<String>,
    pub(super) action_error: Option<String>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            engine: None,
            connection: Connection::Connecting,
            loaded: false,
            store: ContainerStore::default(),
            stats: StatsBoard::default(),
            filter: ContainerFilter::default(),
            selected: None,
            reload_task: None,
            events_task: None,
            stats_tasks: HashMap::new(),
            pending: HashSet::new(),
            action_error: None,
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

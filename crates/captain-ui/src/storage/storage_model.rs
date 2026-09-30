use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use captain_core::Engine;
use captain_core::model::{DiskUsage, EngineInfo};
use captain_core::snapshot::Snapshot;
use captain_core::storage::{DiskBreakdown, ReclaimGroup, ReclaimPlan};
use gpui_kit::*;

use super::StorageEvent;
use crate::engine_host::{host_model, uses_captain};
use crate::workspace::{Page, Workspace};

/// Reads the disk use again this often while connected, for the status bar.
const REFRESH: Duration = Duration::from_secs(10 * 60);

/// The engine's disk use, the cleanup plan, and the groups the user checked. One
/// model serves every window, so the weekly cleanup runs once.
pub struct StorageModel {
    pub(super) workspace: WeakEntity<Workspace>,
    pub(super) engine: Option<Arc<dyn Engine>>,
    pub(super) usage: Option<DiskUsage>,
    pub(super) plan: ReclaimPlan,
    /// The daemon the plan came from. A cleanup removes nothing on another one.
    pub(super) plan_engine: Option<EngineInfo>,
    /// Captain Engine's snapshots. They live on this computer's disk.
    pub(super) snapshots: Vec<Snapshot>,
    /// Free bytes on this computer's disk, when Captain Engine reports it.
    pub(super) host_free: Option<u64>,
    pub(super) error: Option<String>,
    pub(super) checked: Vec<ReclaimGroup>,
    pub(super) snapshot_first: bool,
    /// What the running cleanup does now.
    pub(super) step: Option<SharedString>,
    showing: bool,
    load: Option<Task<()>>,
    _refresh: Task<()>,
    pub(super) _weekly: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<StorageEvent> for StorageModel {}

struct StorageHandle(Entity<StorageModel>);

impl Global for StorageHandle {}

/// Creates the model for `workspace`. Call it once, after Captain Engine's model.
pub fn init(cx: &mut App, workspace: &Entity<Workspace>) {
    let model = cx.new(|cx| StorageModel::new(workspace, cx));
    cx.set_global(StorageHandle(model));
}

/// The app's storage model, once [`init`] ran.
pub fn storage_model(cx: &App) -> Option<Entity<StorageModel>> {
    cx.try_global::<StorageHandle>()
        .map(|handle| handle.0.clone())
}

impl StorageModel {
    fn new(workspace: &Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(workspace, |this: &mut Self, workspace, cx| {
            let (engine, showing) = {
                let workspace = workspace.read(cx);
                (workspace.engine(), workspace.page() == Page::Storage)
            };
            this.follow_engine(engine, cx);
            if showing && !this.showing {
                this.reload(cx);
            }
            this.showing = showing;
        });
        let refresh = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(REFRESH).await;
                if this.update(cx, |model, cx| model.reload(cx)).is_err() {
                    return;
                }
            }
        });
        let mut model = Self {
            workspace: workspace.downgrade(),
            engine: None,
            usage: None,
            plan: ReclaimPlan::default(),
            plan_engine: None,
            snapshots: Vec::new(),
            host_free: None,
            error: None,
            checked: ReclaimGroup::ALL
                .into_iter()
                .filter(|group| group.checked_by_default())
                .collect(),
            snapshot_first: true,
            step: None,
            showing: false,
            load: None,
            _refresh: refresh,
            _weekly: None,
            _subscriptions: vec![observe],
        };
        model.follow_engine(workspace.read(cx).engine(), cx);
        model.start_weekly(cx);
        model
    }

    fn follow_engine(&mut self, engine: Option<Arc<dyn Engine>>, cx: &mut Context<Self>) {
        let same = match (&self.engine, &engine) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        if same {
            return;
        }
        self.engine = engine;
        self.usage = None;
        self.plan = ReclaimPlan::default();
        self.plan_engine = None;
        self.error = None;
        self.load = None;
        self.reload(cx);
    }

    /// Reads the disk use, and Captain Engine's snapshots, again.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            cx.notify();
            return;
        };
        let snapshots = uses_captain(cx)
            .then(|| host_model(cx))
            .flatten()
            .and_then(|host| host.read(cx).snapshots())
            .map(|store| store.list());
        self.load = Some(cx.spawn(async move |this, cx| {
            let (usage, info) = futures::join!(engine.disk_usage(), engine.info());
            let list = match snapshots {
                Some(list) => list.await.ok(),
                None => None,
            };
            this.update(cx, |model, cx| {
                model.load = None;
                match usage {
                    Ok(usage) => {
                        model.plan = ReclaimPlan::new(&usage, now());
                        model.plan_engine = info.ok();
                        model.usage = Some(usage);
                        model.error = None;
                    }
                    Err(error) => {
                        tracing::warn!(%error, "could not read disk use");
                        model.error = Some(format!("Could not read the disk use: {error}"));
                    }
                }
                model.host_free = list.as_ref().and_then(|list| list.free_bytes);
                model.snapshots = list.map(|list| list.snapshots).unwrap_or_default();
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    pub fn usage(&self) -> Option<&DiskUsage> {
        self.usage.as_ref()
    }

    pub fn plan(&self) -> &ReclaimPlan {
        &self.plan
    }

    pub fn is_loading(&self) -> bool {
        self.load.is_some()
    }

    pub fn is_checked(&self, group: ReclaimGroup) -> bool {
        self.checked.contains(&group)
    }

    pub fn toggle(&mut self, group: ReclaimGroup, cx: &mut Context<Self>) {
        match self.checked.iter().position(|g| *g == group) {
            Some(ix) => {
                self.checked.remove(ix);
            }
            None => self.checked.push(group),
        }
        cx.notify();
    }

    pub fn set_snapshot_first(&mut self, on: bool, cx: &mut Context<Self>) {
        self.snapshot_first = on;
        cx.notify();
    }

    /// The bytes the checked groups free.
    pub fn selected_bytes(&self) -> u64 {
        self.plan.total_bytes(&self.checked)
    }

    /// The bytes the groups that start checked free, for the status bar.
    pub fn default_bytes(&self) -> u64 {
        let groups: Vec<_> = ReclaimGroup::ALL
            .into_iter()
            .filter(|group| group.checked_by_default())
            .collect();
        self.plan.total_bytes(&groups)
    }

    /// The disk bar. With Captain Engine, the snapshots and the disk size count.
    pub fn breakdown(&self, cx: &App) -> Option<DiskBreakdown> {
        let usage = self.usage.as_ref()?;
        let captain = uses_captain(cx);
        let snapshots = captain.then(|| {
            self.snapshots
                .iter()
                .map(|s| s.metadata.disk_allocated)
                .sum()
        });
        let capacity = captain
            .then(|| host_model(cx).map(|host| host.read(cx).resources().disk_bytes))
            .flatten();
        Some(DiskBreakdown::new(usage, snapshots, capacity))
    }
}

/// The current Unix time in seconds.
pub(super) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as i64)
}

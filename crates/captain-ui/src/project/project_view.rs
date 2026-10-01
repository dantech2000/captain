use std::collections::HashMap;
use std::time::{Duration, Instant};

use captain_core::format::bytes_label;
use captain_core::known_projects::KnownProject;
use captain_core::model::{
    ComposeProject, ContainerDetail, ContainerState, EngineEvent, EventKind,
};
use captain_core::problems::MemoryRaises;
use captain_core::project_map::StagedChanges;
use captain_core::store::{GroupKey, compose_project};
use gpui_kit::*;

use super::files::FilesState;
use super::group_info::{is_sandbox, service_name};
use super::log_view::ProjectLogView;
use super::map::MapState;
use super::tasks::TaskState;
use super::view_tabs::ProjectTab;
use super::{ProjectNotice, page};
use crate::engine_host::{HostModel, host_model};
use crate::new_project::known_model;
use crate::workspace::Workspace;

/// Exits count toward "3 times in 2 min" for this long.
const EXIT_WINDOW: Duration = Duration::from_secs(120);

/// The Project page: the header, the Open row, a card per service, the tasks, and
/// one log for all services.
pub struct ProjectView {
    pub(super) workspace: Entity<Workspace>,
    pub(super) host: Option<Entity<HostModel>>,
    focus: Option<GroupKey>,
    /// The last Compose project seen for the entry. It stays after `down` removed
    /// the containers, so the header can offer Up.
    pub(super) project: Option<ComposeProject>,
    /// Each container's `inspect` result, and the state it was taken in.
    pub(super) details: HashMap<String, (ContainerState, ContainerDetail)>,
    inspecting: HashMap<String, (ContainerState, Task<()>)>,
    /// When each service's containers exited, the newest last.
    exits: HashMap<String, Vec<Instant>>,
    pub(super) tasks: TaskState,
    pub(super) log: Entity<ProjectLogView>,
    pub(super) tab: ProjectTab,
    pub(super) map: MapState,
    /// Changes that wait for Apply, for all entries. They stay when the page shows
    /// another entry, and leave with their container.
    pub(super) staged: StagedChanges,
    /// The Files tab: the Dockerfile list and the open files.
    pub(super) files: FilesState,
    /// The memory limits raised from a card, so the card does not offer a second
    /// raise for the same out-of-memory kill.
    raises: MemoryRaises,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ProjectNotice> for ProjectView {}

impl ProjectView {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let host = host_model(cx);
        let mut subscriptions = vec![
            cx.observe(&workspace, |this, _, cx| {
                this.follow(cx);
                cx.notify();
            }),
            cx.subscribe(&workspace, |this, _, event: &EngineEvent, cx| {
                this.record(event, cx)
            }),
        ];
        subscriptions.extend(
            host.as_ref()
                .map(|host| cx.observe(host, |_, _, cx| cx.notify())),
        );
        let known = known_model(cx);
        subscriptions.push(cx.observe(&known, |this, _, cx| {
            this.follow(cx);
            cx.notify();
        }));
        let log = cx.new(|cx| ProjectLogView::new(workspace.clone(), cx));
        let mut view = Self {
            workspace,
            host,
            focus: None,
            project: None,
            details: HashMap::new(),
            inspecting: HashMap::new(),
            exits: HashMap::new(),
            tasks: TaskState::default(),
            log,
            tab: ProjectTab::default(),
            map: MapState::default(),
            staged: StagedChanges::default(),
            files: FilesState::default(),
            raises: MemoryRaises::default(),
            _subscriptions: subscriptions,
        };
        view.follow(cx);
        view
    }

    /// Shows the Files tab, for example for a project the user just opened.
    pub fn show_files(&mut self, cx: &mut Context<Self>) {
        self.tab = ProjectTab::Files;
        cx.notify();
    }

    /// Exits of `service` in the last two minutes.
    pub(super) fn recent_exits(&self, service: &str) -> usize {
        self.exits.get(service).map_or(0, |times| {
            times.iter().filter(|at| at.elapsed() < EXIT_WINDOW).count()
        })
    }

    /// The limit container `id` ran out of, when its last run was killed at the
    /// current limit.
    pub(super) fn oom_limit(&self, id: &str) -> Option<u64> {
        let (_, detail) = self.details.get(id)?;
        self.raises.oom_limit(detail)
    }

    /// Starts over for a new entry, keeps the Compose project, and inspects each
    /// container whose state changed.
    pub(super) fn follow(&mut self, cx: &mut Context<Self>) {
        let workspace = self.workspace.read(cx);
        let focus = workspace.focus().cloned();
        if focus != self.focus {
            self.focus = focus.clone();
            self.project = None;
            self.details.clear();
            self.inspecting.clear();
            self.exits.clear();
            self.tasks.forget_list();
            self.map.draft = None;
            self.files.forget_list();
            if self.tab == ProjectTab::Files && !matches!(focus, Some(GroupKey::Project(_))) {
                self.tab = ProjectTab::Overview;
            }
            if self.tab == ProjectTab::Map {
                self.load_volumes(cx);
            }
        }
        let workspace = self.workspace.read(cx);
        if workspace.is_loaded() {
            let store = workspace.store();
            self.staged.retain_containers(|id| store.find(id).is_some());
        }
        let known = known_model(cx).read(cx).projects().to_vec();
        let workspace = self.workspace.read(cx);
        if let Some(GroupKey::Project(name)) = &focus
            && let Some(project) = compose_project(workspace.store().containers(), name)
                .or_else(|| self.project.clone())
                .or_else(|| {
                    known
                        .iter()
                        .find(|k| &k.name == name)
                        .map(KnownProject::compose_project)
                })
        {
            let stale = self.tasks.stale(&project);
            let compose = workspace.has_project_runner();
            self.project = Some(project.clone());
            if stale && compose {
                self.load_tasks(&project, cx);
            }
        }
        let workspace = self.workspace.read(cx);
        let (Some(group), Some(engine)) = (workspace.focused_group(), workspace.engine()) else {
            return;
        };
        self.details
            .retain(|id, _| group.containers.iter().any(|c| &c.id == id));
        for container in group.containers.iter().filter(|c| !is_sandbox(c)) {
            let state = container.state;
            let known = self.details.get(&container.id).map(|(s, _)| *s);
            let asked = self.inspecting.get(&container.id).map(|(s, _)| *s);
            if known == Some(state) || asked == Some(state) {
                continue;
            }
            let id = container.id.clone();
            let inspect = engine.inspect_container(&id);
            let task = cx.spawn(async move |this, cx| {
                let result = inspect.await;
                this.update(cx, |this, cx| {
                    this.inspecting.remove(&id);
                    if let Ok(detail) = result {
                        this.details.insert(id, (state, detail));
                        cx.notify();
                    }
                })
                .ok();
            });
            self.inspecting.insert(container.id.clone(), (state, task));
        }
    }

    /// Counts the exits of the shown containers.
    fn record(&mut self, event: &EngineEvent, cx: &mut Context<Self>) {
        if event.kind != EventKind::Container || event.action != "die" {
            return;
        }
        let workspace = self.workspace.read(cx);
        let Some(group) = workspace.focused_group() else {
            return;
        };
        if let Some(container) = group
            .containers
            .iter()
            .find(|c| c.id == event.id && !is_sandbox(c))
        {
            let times = self.exits.entry(service_name(container)).or_default();
            times.retain(|at| at.elapsed() < EXIT_WINDOW);
            times.push(Instant::now());
            cx.notify();
        }
    }

    /// Sets the memory limit of container `id` to `bytes`, then inspects it again.
    pub(super) fn raise_memory(
        &mut self,
        id: &str,
        name: &str,
        bytes: u64,
        cx: &mut Context<Self>,
    ) {
        let Some(engine) = self.workspace.read(cx).engine() else {
            return;
        };
        // Recorded now, so a second click does not raise the limit again.
        if let Some((_, detail)) = self.details.get(id) {
            self.raises.record(detail);
        }
        let update = engine.update_memory(id, bytes);
        let (id, name) = (id.to_string(), name.to_string());
        cx.spawn(async move |this, cx| {
            let result = update.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok(()) => {
                        this.details.remove(&id);
                        this.follow(cx);
                        cx.emit(ProjectNotice::MemoryRaised {
                            name,
                            limit: bytes_label(bytes),
                        });
                    }
                    Err(error) => {
                        this.raises.forget(&id);
                        cx.emit(ProjectNotice::Failed {
                            title: format!("Cannot change the memory limit of {name}"),
                            error: error.to_string(),
                        })
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

impl Render for ProjectView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.prepare_files(window, cx);
        page::render(self, cx)
    }
}

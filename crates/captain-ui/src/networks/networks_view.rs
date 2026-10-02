use std::collections::HashSet;
use std::sync::Arc;

use captain_core::Engine;
use captain_core::model::EngineEvent;
use captain_core::store::changes_network_list;
use captain_core::store::{NetworkStore, UsageFilter};
use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::detail::LoadedDetail;
use super::feed::RELOAD_DEBOUNCE;
use super::{network_inspector, network_list, toolbar};
use crate::theme::Palette;
use crate::widgets::{create_field, inline_error, inline_notice, page_header};
use crate::workspace::Workspace;

/// The Networks page: networks grouped by Compose project, with create, remove, prune,
/// and a detail panel for the selected network. It keeps its own list and follows
/// network events, so the workspace does not change.
pub struct NetworksView {
    pub(super) workspace: Entity<Workspace>,
    pub(super) engine: Option<Arc<dyn Engine>>,
    /// Counts engine switches. Destructive actions carry the number they were shown
    /// for, and do nothing after a switch.
    pub(super) generation: u64,
    pub(super) store: NetworkStore,
    pub(super) loaded: bool,
    pub(super) filter: UsageFilter,
    /// The ID of the selected network.
    pub(super) selected: Option<String>,
    /// IDs of networks that are being removed.
    pub(super) removing: HashSet<String>,
    pub(super) creating: bool,
    pub(super) pruning: bool,
    /// The inspect result of the selected network.
    pub(super) detail: Option<LoadedDetail>,
    pub(super) detail_task: Option<Task<()>>,
    /// Why the name in the field is not valid.
    pub(super) name_hint: Option<SharedString>,
    /// The last failed load or action.
    pub(super) error: Option<String>,
    /// The result of the last prune.
    pub(super) notice: Option<String>,
    pub(super) input: Option<Entity<InputState>>,
    pub(super) reload_task: Option<Task<()>>,
    pub(super) subscriptions: Vec<Subscription>,
}

impl NetworksView {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&workspace, |this, workspace, cx| {
            this.follow_engine(workspace.read(cx).engine(), cx);
            cx.notify();
        });
        let events = cx.subscribe(&workspace, |this, _, event: &EngineEvent, cx| {
            if changes_network_list(event) {
                this.reload(RELOAD_DEBOUNCE, cx);
            }
        });
        let engine = workspace.read(cx).engine();
        let mut view = Self {
            workspace,
            engine: None,
            generation: 0,
            store: NetworkStore::default(),
            loaded: false,
            filter: UsageFilter::default(),
            selected: None,
            removing: HashSet::new(),
            creating: false,
            pruning: false,
            detail: None,
            detail_task: None,
            name_hint: None,
            error: None,
            notice: None,
            input: None,
            reload_task: None,
            subscriptions: vec![observe, events],
        };
        view.follow_engine(engine, cx);
        view
    }
}

impl Render for NetworksView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let input = self.input(window, cx);
        let summary = if self.loaded {
            self.store.summary()
        } else {
            "Loading...".into()
        };
        let tools = toolbar::render(self, cx, &palette);
        let create = cx.listener(|this, _: &ClickEvent, window, cx| this.create(window, cx));
        let workspace = self.workspace.read(cx);
        let panel = self
            .selected
            .as_deref()
            .and_then(|id| self.store.find(id))
            .map(|network| {
                let detail = self.detail_of(&network.id);
                network_inspector::render(network, detail, &self.workspace, workspace, &palette)
            });

        let main = div()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .child(page_header(
                "networks-header",
                "Networks",
                summary,
                Some(tools.into_any_element()),
                &palette,
            ))
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .px(px(24.))
                    .pb(px(12.))
                    .child(create_field(
                        "create-network",
                        &input,
                        self.name_hint.clone(),
                        self.creating,
                        &palette,
                        create,
                    ))
                    .children(
                        self.error
                            .clone()
                            .map(|error| inline_error(error, &palette)),
                    )
                    .children(
                        self.notice
                            .clone()
                            .map(|notice| inline_notice(notice, &palette)),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .child(network_list::render(self, cx, &palette)),
            );
        div().size_full().flex().child(main).children(panel)
    }
}

use std::collections::HashSet;
use std::sync::Arc;

use captain_core::Engine;
use captain_core::model::EngineEvent;
use captain_core::store::changes_volume_list;
use captain_core::store::{MultiSelection, UsageFilter, VolumeStore};
use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::feed::RELOAD_DEBOUNCE;
use super::users::VolumeUsers;
use super::{toolbar, volume_inspector, volume_list};
use crate::theme::Palette;
use crate::widgets::{
    ButtonTone, create_field, inline_error, inline_notice, page_header, selection_bar, text_button,
};
use crate::workspace::{Connection, Workspace};

/// The Volumes page: volumes grouped by Compose project, with create, remove, prune,
/// and a detail panel for the selected volume. It keeps its own list and follows
/// volume events, so the workspace does not change.
pub struct VolumesView {
    pub(super) workspace: Entity<Workspace>,
    pub(super) engine: Option<Arc<dyn Engine>>,
    /// Counts engine switches. Destructive actions carry the number they were shown
    /// for, and do nothing after a switch.
    pub(super) generation: u64,
    pub(super) store: VolumeStore,
    pub(super) loaded: bool,
    pub(super) filter: UsageFilter,
    /// The name of the selected volume.
    pub(super) selected: Option<String>,
    /// The volumes selected for a bulk delete. It holds the selected volume too.
    pub(super) checked: MultiSelection,
    /// Volumes that are being removed.
    pub(super) removing: HashSet<String>,
    pub(super) creating: bool,
    pub(super) pruning: bool,
    /// The containers that use the selected volume.
    pub(super) users: Option<VolumeUsers>,
    pub(super) users_task: Option<Task<()>>,
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

impl VolumesView {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&workspace, |this, workspace, cx| {
            this.follow_engine(workspace.read(cx).engine(), cx);
            // An extension's `navigate.viewVolume` asks for a volume.
            let revealed = workspace.update(cx, |workspace, _| workspace.take_revealed_volume());
            if let Some(name) = revealed
                && this.selected.as_deref() != Some(name.as_str())
            {
                this.select(name, cx);
            }
            cx.notify();
        });
        let events = cx.subscribe(&workspace, |this, _, event: &EngineEvent, cx| {
            if changes_volume_list(event) {
                this.reload(RELOAD_DEBOUNCE, cx);
            }
        });
        let engine = workspace.read(cx).engine();
        let mut view = Self {
            workspace,
            engine: None,
            generation: 0,
            store: VolumeStore::default(),
            loaded: false,
            filter: UsageFilter::default(),
            selected: None,
            checked: MultiSelection::default(),
            removing: HashSet::new(),
            creating: false,
            pruning: false,
            users: None,
            users_task: None,
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

impl Render for VolumesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let input = self.input(window, cx);
        let failed = match self.workspace.read(cx).connection() {
            Connection::Failed(error) => Some(error.to_string()),
            _ => None,
        };
        let summary = if self.loaded {
            self.store.summary()
        } else {
            "Loading...".into()
        };
        let tools = toolbar::render(self, cx, &palette);
        let create = cx.listener(|this, _: &ClickEvent, window, cx| this.create(window, cx));
        let bulk = self.checked.is_bulk().then(|| {
            let delete = text_button(
                "bulk-delete-volumes",
                "Delete",
                ButtonTone::Danger,
                true,
                &palette,
                cx.listener(|this, _, window, cx| this.confirm_bulk_delete(window, cx)),
            );
            selection_bar(
                self.checked.len(),
                vec![delete],
                &palette,
                cx.listener(|this, _, _, cx| this.clear_bulk(cx)),
            )
        });
        let workspace = self.workspace.read(cx);
        let panel = self
            .selected
            .as_deref()
            .and_then(|name| self.store.find(name))
            .map(|volume| {
                let users = self.users_of(&volume.name);
                volume_inspector::render(volume, users, &self.workspace, workspace, &palette)
            });

        let main = div()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .child(page_header(
                "volumes-header",
                "Volumes",
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
                        "create-volume",
                        &input,
                        self.name_hint.clone(),
                        self.creating,
                        &palette,
                        create,
                    ))
                    .children(
                        failed
                            .or(self.error.clone())
                            .map(|error| inline_error(error, &palette)),
                    )
                    .children(
                        self.notice
                            .clone()
                            .map(|notice| inline_notice(notice, &palette)),
                    ),
            )
            .children(bulk)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .child(volume_list::render(self, cx, &palette)),
            );
        div().size_full().flex().child(main).children(panel)
    }
}

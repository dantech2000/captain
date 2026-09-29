use std::sync::Arc;
use std::time::Duration;

use captain_core::model::Container;
use captain_core::{Engine, EngineError};
use futures::StreamExt;
use gpui_kit::component::label::Label;
use gpui_kit::component::table::{DataTable, TableState};
use gpui_kit::component::{ActiveTheme, v_flex};
use gpui_kit::*;

use super::container_table::ContainerTable;
use super::{empty_state, error_state, header};

/// Events often come in bursts, for example `docker compose up`. Wait this long after
/// the last event before reloading the list.
const RELOAD_DEBOUNCE: Duration = Duration::from_millis(150);

enum Load {
    Waiting,
    Ready,
    Failed(EngineError),
}

/// The Containers page: a live table of every container on the engine.
pub struct ContainersView {
    table: Entity<TableState<ContainerTable>>,
    engine: Option<Arc<dyn Engine>>,
    load: Load,
    /// Replacing this task cancels a pending reload, which gives the debounce.
    reload_task: Option<Task<()>>,
    events_task: Option<Task<()>>,
}

impl ContainersView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let table = cx.new(|cx| TableState::new(ContainerTable::default(), window, cx));
        Self {
            table,
            engine: None,
            load: Load::Waiting,
            reload_task: None,
            events_task: None,
        }
    }

    /// Starts showing containers from `engine` and following its events.
    pub fn attach(&mut self, engine: Arc<dyn Engine>, cx: &mut Context<Self>) {
        self.engine = Some(engine);
        self.reload(Duration::ZERO, cx);
        self.watch_events(cx);
    }

    pub fn fail(&mut self, error: EngineError, cx: &mut Context<Self>) {
        self.load = Load::Failed(error);
        self.events_task = None;
        self.reload_task = None;
        cx.notify();
    }

    fn reload(&mut self, delay: Duration, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            if !delay.is_zero() {
                cx.background_executor().timer(delay).await;
            }
            let result = engine.list_containers().await;
            this.update(cx, |this, cx| this.apply(result, cx)).ok();
        }));
    }

    fn apply(&mut self, result: Result<Vec<Container>, EngineError>, cx: &mut Context<Self>) {
        match result {
            Ok(containers) => {
                self.table.update(cx, |table, cx| {
                    table.delegate_mut().replace(containers);
                    cx.notify();
                });
                self.load = Load::Ready;
                cx.notify();
            }
            Err(error) => self.fail(error, cx),
        }
    }

    fn watch_events(&mut self, cx: &mut Context<Self>) {
        let Some(engine) = self.engine.clone() else {
            return;
        };
        let mut events = engine.events();
        self.events_task = Some(cx.spawn(async move |this, cx| {
            while let Some(event) = events.next().await {
                let updated = match event {
                    Ok(event) if event.changes_container_list() => {
                        this.update(cx, |this, cx| this.reload(RELOAD_DEBOUNCE, cx))
                    }
                    Ok(_) => continue,
                    Err(error) => this.update(cx, |this, cx| this.fail(error, cx)),
                };
                if updated.is_err() {
                    return;
                }
            }
            let closed = EngineError::Unreachable("the engine closed the event stream".into());
            this.update(cx, |this, cx| this.fail(closed, cx)).ok();
        }));
    }
}

impl Render for ContainersView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = match &self.load {
            Load::Waiting => Label::new("Connecting to Docker...")
                .text_color(cx.theme().muted_foreground)
                .into_any_element(),
            Load::Failed(error) => error_state::render(error, cx).into_any_element(),
            Load::Ready if self.table.read(cx).delegate().store().is_empty() => {
                empty_state::render(cx).into_any_element()
            }
            Load::Ready => DataTable::new(&self.table).stripe(true).into_any_element(),
        };
        let store = matches!(self.load, Load::Ready)
            .then(|| self.table.read(cx).delegate().store().clone());

        v_flex()
            .size_full()
            .child(header::render(store.as_ref(), cx))
            .child(div().flex_1().min_h_0().child(body))
    }
}

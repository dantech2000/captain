use std::time::{SystemTime, UNIX_EPOCH};

use captain_core::model::EngineEvent;
use captain_core::store::ImageFilter;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::pull_status::message;
use super::{
    ImagesState, column_header, empty_state, header, image_row, inspector, pull_status,
    started_notice, toolbar,
};
use crate::theme::Palette;
use crate::workspace::{Connection, Page, Workspace};

/// The Images page: the image list with a filter, pull, remove, and prune, and the
/// inspector for the selected image on the right.
pub struct ImagesView {
    workspace: Entity<Workspace>,
    state: Entity<ImagesState>,
    /// The pull field. It needs a window, so the first render creates it.
    input: Option<Entity<InputState>>,
    _subscriptions: Vec<Subscription>,
}

impl ImagesView {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let state = cx.new(|_| ImagesState::new());
        let observe_workspace = cx.observe(&workspace, |this, workspace, cx| {
            this.attach_engine(cx);
            // An extension's `navigate.viewImage` asks for an image.
            if let Some(id) = workspace.update(cx, |workspace, _| workspace.take_revealed_image()) {
                this.state.update(cx, |state, cx| state.select(id, cx));
            }
        });
        let observe_state = cx.observe(&state, |this, state, cx| {
            let count = state.read(cx).store.len();
            this.workspace.update(cx, |workspace, cx| {
                workspace.set_page_count(Page::Images, count, cx)
            });
            cx.notify();
        });
        let events = cx.subscribe(&workspace, |this, _, event: &EngineEvent, cx| {
            this.state
                .update(cx, |state, cx| state.on_engine_event(event, cx));
        });
        let mut view = Self {
            workspace,
            state,
            input: None,
            _subscriptions: vec![observe_workspace, observe_state, events],
        };
        view.attach_engine(cx);
        view
    }

    /// Hands the workspace engine to the image state once it has connected.
    fn attach_engine(&mut self, cx: &mut Context<Self>) {
        let workspace = self.workspace.read(cx);
        if let Some(engine) = workspace.engine() {
            let builder = workspace.image_builder();
            self.state
                .update(cx, |state, cx| state.attach(engine, builder, cx));
        } else {
            self.state.update(cx, |state, cx| state.detach(cx));
        }
        cx.notify();
    }

    fn input(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Entity<InputState> {
        if let Some(input) = &self.input {
            return input.clone();
        }
        let input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Image to pull, for example nginx:alpine")
        });
        let enter = cx.subscribe_in(&input, window, |this, input, event, window, cx| {
            if let InputEvent::PressEnter { .. } = event {
                toolbar::start_pull(&this.state, input, window, cx);
            }
        });
        self._subscriptions.push(enter);
        self.input = Some(input.clone());
        input
    }
}

impl Render for ImagesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let input = self.input(window, cx);
        let state = self.state.read(cx);

        let body = match self.workspace.read(cx).connection() {
            Connection::Failed(error) => centered(
                message(&format!("Cannot reach the engine: {error}"), palette.red),
                &palette,
            ),
            _ if !state.is_loaded() => match state.load_error() {
                Some(error) => centered(message(error, palette.red), &palette),
                None => centered(div().child("Loading images..."), &palette),
            },
            _ => self.list(state, &palette),
        };

        let messages =
            [
                state.load_error().filter(|_| state.is_loaded()),
                state.error(),
            ]
            .into_iter()
            .flatten()
            .map(|error| message(error, palette.red))
            .chain(state.notice().map(|notice| message(notice, palette.green)))
            .chain(state.started().map(|started| {
                started_notice::render(started, &self.state, &self.workspace, &palette)
            }));

        let page = div()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .child(header::render(&self.state, state, &palette))
            .child(toolbar::render(&self.state, state, &input, &palette))
            .children(pull_status::render(state, &palette))
            .child(
                div()
                    .flex_shrink_0()
                    .px(px(24.))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .children(messages.map(|line| line.pb(px(8.)))),
            )
            .child(div().flex_1().min_h_0().child(body));
        let inspector = state
            .selected()
            .map(|image| inspector::render(image, &self.state, state, &palette));

        div().size_full().flex().child(page).children(inspector)
    }
}

impl ImagesView {
    fn list(&self, state: &ImagesState, palette: &Palette) -> AnyElement {
        let images = state.store().filtered(state.filter());
        let now = unix_now();
        let content = if images.is_empty() {
            empty_state::render(state.filter() != ImageFilter::All, palette).into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .p(px(4.))
                .rounded(px(12.))
                .bg(palette.group)
                .border_1()
                .border_color(palette.sep)
                .children(
                    images
                        .into_iter()
                        .map(|image| image_row::render(image, &self.state, state, now, palette)),
                )
                .into_any_element()
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(column_header::render(palette))
            .child(
                div()
                    .id("image-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(px(12.))
                    .pt(px(10.))
                    .pb(px(16.))
                    .child(content),
            )
            .into_any_element()
    }
}

fn centered(content: Div, palette: &Palette) -> AnyElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .p(px(40.))
        .text_color(palette.text2)
        .child(content)
        .into_any_element()
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as i64)
}

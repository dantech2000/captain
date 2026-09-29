use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::engine_source::{self, DetectedEndpoint};
use super::store::{self, SettingsStore};
use super::{about_section, appearance_section, endpoint_picker, engine_section};
use crate::theme::Palette;
use crate::widgets::{inline_error, page_header};
use crate::workspace::Workspace;

/// The Settings page: appearance, the engine connection, and About.
pub struct SettingsView {
    pub(super) workspace: Entity<Workspace>,
    /// The custom endpoint field. It needs a window, so the first render creates it.
    input: Option<Entity<InputState>>,
    /// Why the custom endpoint is not valid.
    pub(super) hint: Option<SharedString>,
    /// Engines found on this machine. A rescan or a switch refreshes them.
    pub(super) detected: Vec<DetectedEndpoint>,
    subscriptions: Vec<Subscription>,
}

impl SettingsView {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&workspace, |_, _, cx| cx.notify());
        let settings = cx.observe_global::<SettingsStore>(|_, cx| cx.notify());
        let mut view = Self {
            workspace,
            input: None,
            hint: None,
            detected: Vec::new(),
            subscriptions: vec![observe, settings],
        };
        view.rescan(cx);
        view
    }

    /// Looks for engines on this machine again.
    pub(super) fn rescan(&mut self, cx: &mut Context<Self>) {
        self.detected = engine_source::engine_source(cx)
            .map(|source| source.detected())
            .unwrap_or_default();
        cx.notify();
    }

    /// Saves `endpoint` (`None` for discovery) and reconnects.
    pub(super) fn use_engine(&mut self, endpoint: Option<String>, cx: &mut Context<Self>) {
        self.hint = None;
        engine_source::use_engine(&self.workspace, endpoint, cx);
        self.rescan(cx);
    }

    /// Checks the custom endpoint field, then uses it.
    pub(super) fn use_custom(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(input) = self.input.clone() else {
            return;
        };
        let host = input.read(cx).value().trim().to_string();
        let checked = if host.is_empty() {
            Err("Enter an endpoint, for example unix:///var/run/docker.sock.".to_string())
        } else {
            engine_source::engine_source(cx).map_or(
                Err("Captain cannot switch engines here.".into()),
                |source| source.check_endpoint(&host),
            )
        };
        match checked {
            Ok(()) => {
                input.update(cx, |input, cx| input.set_value("", window, cx));
                self.use_engine(Some(host), cx);
            }
            Err(message) => {
                self.hint = Some(message.into());
                cx.notify();
            }
        }
    }

    fn input(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Entity<InputState> {
        if let Some(input) = &self.input {
            return input.clone();
        }
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("unix:///path/to/engine.sock or tcp://host:2375")
        });
        let events = cx.subscribe_in(&input, window, |this, _, event, window, cx| match event {
            InputEvent::PressEnter { .. } => this.use_custom(window, cx),
            InputEvent::Change if this.hint.is_some() => {
                this.hint = None;
                cx.notify();
            }
            _ => {}
        });
        self.subscriptions.push(events);
        self.input = Some(input.clone());
        input
    }
}

impl Render for SettingsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let settings = store::current(cx);
        let input = self.input(window, cx);

        let cards = div()
            .w_full()
            .max_w(px(720.))
            .flex()
            .flex_col()
            .gap(px(22.))
            .children(store::save_error(cx).map(|error| {
                inline_error(
                    format!("Captain cannot save the settings. {error}"),
                    &palette,
                )
            }))
            .child(appearance_section::render(&settings, &palette))
            .child(engine_section::render(self, &settings, &palette, cx))
            .child(endpoint_picker::render(self, &input, &palette, cx))
            .child(about_section::render(&palette));

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(page_header(
                "settings-header",
                "Settings",
                "Appearance, the engine connection, and information about Captain",
                None,
                &palette,
            ))
            .child(
                div()
                    .id("settings-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(px(24.))
                    .pt(px(4.))
                    .pb(px(24.))
                    .child(cards),
            )
    }
}

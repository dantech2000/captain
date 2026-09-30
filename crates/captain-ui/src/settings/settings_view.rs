use captain_core::docker_context::ContextList;
use captain_core::kubernetes::KubernetesSettings;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::admin_access::AdminAccess;
use super::cli_tools_state::CliToolsCard;
use super::context_actions::ContextChange;
use super::engine_source::{self, DetectedEndpoint};
use super::kube_form::KubeForm;
use super::store::{self, SettingsStore};
use super::{
    about_section, appearance_section, engine_section, file_section, kubernetes_section,
    startup_section, terminal_section,
};
use crate::engine_host::{HostModel, host_model};
use crate::kubernetes::{KubernetesModel, kubernetes_model};
use crate::theme::Palette;
use crate::widgets::inline_error;
use crate::workspace::Workspace;

/// The one-page Settings: appearance, the engine, Kubernetes, startup, the
/// terminal, the settings file, and About. See feature 0037.
pub struct SettingsView {
    pub(super) workspace: Entity<Workspace>,
    /// Captain Engine, when the app has one.
    pub(super) host: Option<Entity<HostModel>>,
    /// Free bytes on Captain Engine's disk, read in the background.
    pub(super) free_disk: Option<u64>,
    /// The k3s cluster, when Captain Engine has one.
    pub(super) kubernetes: Option<Entity<KubernetesModel>>,
    /// The Kubernetes version picker. The first render creates it.
    pub(super) kube_form: Option<KubeForm>,
    /// The saved settings the Kubernetes form shows.
    pub(super) kube_form_source: Option<KubernetesSettings>,
    /// The remote host field of the engines sheet. It needs a window, so the first
    /// render creates it.
    pub(super) input: Option<Entity<InputState>>,
    /// Why the custom endpoint is not valid.
    pub(super) hint: Option<SharedString>,
    /// Engines found on this machine. A rescan or a switch refreshes them.
    pub(super) detected: Vec<DetectedEndpoint>,
    /// The Docker CLI contexts. A rescan or a context change refreshes them.
    pub(super) contexts: ContextList,
    pub(super) context_change: ContextChange,
    /// Why the last change to the login item failed.
    pub(super) login_error: Option<SharedString>,
    pub(super) admin_access: AdminAccess,
    /// The terminal setup's state. A background read fills it.
    pub(super) cli_tools: CliToolsCard,
    /// True when the terminal sheet shows the plain shell PATH line next to the
    /// home-manager one.
    pub(super) show_plain_line: bool,
    pub(super) subscriptions: Vec<Subscription>,
}

impl SettingsView {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&workspace, |_, _, cx| cx.notify());
        let settings = cx.observe_global::<SettingsStore>(|_, cx| cx.notify());
        let host = host_model(cx);
        let kubernetes = kubernetes_model(cx);
        let mut subscriptions = vec![observe, settings];
        subscriptions.extend(
            host.as_ref()
                .map(|host| cx.observe(host, |_, _, cx| cx.notify())),
        );
        subscriptions.extend(
            kubernetes
                .as_ref()
                .map(|model| cx.observe(model, |_, _, cx| cx.notify())),
        );
        let mut view = Self {
            workspace,
            host,
            free_disk: None,
            kubernetes,
            kube_form: None,
            kube_form_source: None,
            input: None,
            hint: None,
            detected: Vec::new(),
            contexts: ContextList::default(),
            context_change: ContextChange::default(),
            login_error: None,
            admin_access: AdminAccess::default(),
            cli_tools: CliToolsCard::default(),
            show_plain_line: false,
            subscriptions,
        };
        view.rescan(cx);
        view.refresh_tools(cx);
        view.read_free_disk(cx);
        view
    }

    /// Reads the free space on Captain Engine's disk in the background.
    fn read_free_disk(&mut self, cx: &mut Context<Self>) {
        let Some(task) = self.host.as_ref().map(|host| host.read(cx).free_disk(cx)) else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let free = task.await;
            this.update(cx, |view, cx| {
                view.free_disk = free;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Looks for engines on this machine again.
    pub(super) fn rescan(&mut self, cx: &mut Context<Self>) {
        let source = engine_source::engine_source(cx);
        self.detected = source
            .as_ref()
            .map(|source| source.detected())
            .unwrap_or_default();
        self.contexts = source.map(|source| source.contexts()).unwrap_or_default();
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
                .placeholder("unix:///path/to/engine.sock or ssh://user@host")
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
        self.input(window, cx);
        let versions = self
            .kubernetes
            .as_ref()
            .map(|model| model.read(cx).versions().clone());
        let saved = settings.kubernetes.version.clone();
        if let (Some(form), Some(versions)) =
            (self.kube_form(&settings.kubernetes, window, cx), versions)
        {
            form.show_versions(&versions, saved.as_deref(), window, cx);
        }
        let kubernetes =
            self.kubernetes
                .clone()
                .zip(self.host.clone())
                .and_then(|(model, host)| {
                    let form = self.kube_form.as_ref()?;
                    kubernetes_section::render(&model, &host, form, &palette, cx)
                });

        let column = div()
            .w(px(760.))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(px(14.))
            .py(px(34.))
            .child(
                div()
                    .mb(px(4.))
                    .text_size(px(26.))
                    .font_weight(FontWeight::EXTRA_BOLD)
                    .child("Settings"),
            )
            .children(store::save_error(cx).map(|error| {
                inline_error(
                    format!("Captain cannot save the settings. {error}"),
                    &palette,
                )
            }))
            .child(appearance_section::render(&settings, &palette))
            .child(engine_section::render(self, &palette, cx))
            .children(kubernetes)
            .children(startup_section::render(self, &settings, &palette, cx))
            .child(terminal_section::render(self, &settings, &palette, cx))
            .child(file_section::render(&palette))
            .child(about_section::render(self, &palette, cx));

        div()
            .id("settings-scroll")
            .size_full()
            .overflow_y_scroll()
            .flex()
            .justify_center()
            .px(px(24.))
            .text_color(palette.text)
            .child(column)
    }
}

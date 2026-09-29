use captain_core::daemon::DaemonSettings;
use captain_core::docker_context::ContextList;
use captain_core::kubernetes::KubernetesSettings;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::admin_access_section::{self, AdminAccess};
use super::context_actions::ContextChange;
use super::daemon_form::DaemonForm;
use super::engine_source::{self, DetectedEndpoint};
use super::kube_form::KubeForm;
use super::store::{self, SettingsStore};
use super::{
    about_section, appearance_section, behavior_section, captain_engine_section, daemon_section,
    endpoint_picker, engine_section, kubernetes_section,
};
use crate::engine_host::{HostModel, host_model};
use crate::kubernetes::{KubernetesModel, kubernetes_model};
use crate::theme::Palette;
use crate::widgets::{inline_error, page_header};
use crate::workspace::Workspace;

/// The Settings page: appearance, behavior, the engine connection, and About.
pub struct SettingsView {
    pub(super) workspace: Entity<Workspace>,
    /// Captain Engine, when the app has one.
    pub(super) host: Option<Entity<HostModel>>,
    /// The Docker daemon fields. They need a window, so the first render creates them.
    pub(super) daemon_form: Option<DaemonForm>,
    /// The saved settings the daemon form shows. A change from elsewhere, such as a
    /// snapshot restore, rebuilds the form so Save cannot write old values back.
    daemon_form_source: Option<DaemonSettings>,
    /// The k3s cluster, when Captain Engine has one.
    pub(super) kubernetes: Option<Entity<KubernetesModel>>,
    /// The Kubernetes version picker and port. The first render creates them.
    pub(super) kube_form: Option<KubeForm>,
    /// The saved settings the Kubernetes form shows.
    pub(super) kube_form_source: Option<KubernetesSettings>,
    /// The custom endpoint field. It needs a window, so the first render creates it.
    input: Option<Entity<InputState>>,
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
            daemon_form: None,
            daemon_form_source: None,
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
            subscriptions,
        };
        view.rescan(cx);
        view
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
        let input = self.input(window, cx);
        if self.daemon_form_source.as_ref() != Some(&settings.engine_daemon) {
            self.daemon_form = Some(DaemonForm::new(&settings.engine_daemon, window, cx));
            self.daemon_form_source = Some(settings.engine_daemon.clone());
        }
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

        let cards =
            div()
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
                .children(behavior_section::render(self, &settings, &palette, cx))
                .children(
                    self.host
                        .as_ref()
                        .map(|host| captain_engine_section::render(host, &settings, &palette, cx)),
                )
                .children(
                    self.host
                        .as_ref()
                        .zip(self.daemon_form.as_ref())
                        .and_then(|(host, form)| daemon_section::render(host, form, &palette, cx)),
                )
                .children(self.kubernetes.clone().zip(self.host.clone()).and_then(
                    |(model, host)| {
                        let form = self.kube_form.as_ref()?;
                        kubernetes_section::render(&model, &host, form, &palette, cx)
                    },
                ))
                .children(admin_access_section::render(self, &palette, cx))
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
                "Appearance, behavior, Captain Engine, the engine connection, and information about Captain",
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

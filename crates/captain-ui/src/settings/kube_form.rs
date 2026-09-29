//! The fields of the Kubernetes card: the version picker and the port. They need a
//! window, so the Settings page creates them on its first render. See feature 0024.

use captain_core::kubernetes::{K3sVersion, KubernetesSettings, VersionList};
use gpui_kit::component::input::InputState;
use gpui_kit::component::select::{SelectEvent, SelectState};
use gpui_kit::*;

use super::{SettingsView, kube_dialogs};
use crate::kubernetes::KubernetesModel;

/// The version labels, such as `v1.36.4+k3s1 (stable)`.
pub type VersionSelect = SelectState<Vec<SharedString>>;

pub struct KubeForm {
    pub version: Entity<VersionSelect>,
    pub port: Entity<InputState>,
    /// The versions behind the labels, in the same order.
    versions: Vec<K3sVersion>,
    labels: Vec<SharedString>,
    /// Why the last Apply was refused.
    pub error: Option<SharedString>,
}

impl KubeForm {
    pub fn new(settings: &KubernetesSettings, window: &mut Window, cx: &mut App) -> Self {
        let port = settings.port.to_string();
        Self {
            version: cx.new(|cx| SelectState::new(Vec::new(), None, window, cx)),
            port: cx.new(|cx| InputState::new(window, cx).default_value(port)),
            versions: Vec::new(),
            labels: Vec::new(),
            error: None,
        }
    }

    /// Shows `list` and the saved version, if the list changed.
    pub fn show_versions(
        &mut self,
        list: &VersionList,
        saved: Option<&str>,
        window: &mut Window,
        cx: &mut App,
    ) {
        let mut versions = list.versions.clone();
        if let Some(saved) = saved.and_then(|saved| saved.parse().ok())
            && !versions.contains(&saved)
        {
            versions.insert(0, saved);
        }
        if versions == self.versions {
            return;
        }
        let labels: Vec<SharedString> = versions.iter().map(|v| list.label(v).into()).collect();
        let selected = saved
            .and_then(|saved| versions.iter().position(|v| v.as_str() == saved))
            .map(|index| labels[index].clone());
        self.versions = versions;
        self.labels = labels.clone();
        self.version.update(cx, |select, cx| {
            select.set_items(labels, window, cx);
            if let Some(label) = selected {
                select.set_selected_value(&label, window, cx);
            }
        });
    }

    fn version_for(&self, label: &SharedString) -> Option<K3sVersion> {
        let index = self.labels.iter().position(|shown| shown == label)?;
        self.versions.get(index).cloned()
    }
}

impl SettingsView {
    /// The form, created on first use. It follows the picker.
    pub(super) fn kube_form(
        &mut self,
        settings: &KubernetesSettings,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<&mut KubeForm> {
        if self.kube_form.is_none() && self.kubernetes.is_some() {
            let form = KubeForm::new(settings, window, cx);
            let picked = cx.subscribe_in(
                &form.version,
                window,
                |view, _, event: &SelectEvent<Vec<SharedString>>, window, cx| {
                    let SelectEvent::Confirm(Some(label)) = event else {
                        return;
                    };
                    view.pick_version(label.clone(), window, cx);
                },
            );
            self.subscriptions.push(picked);
            self.kube_form = Some(form);
        }
        self.kube_form.as_mut()
    }

    /// Saves the picked version. A downgrade of a running cluster asks first,
    /// because it needs a reset.
    fn pick_version(&mut self, label: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(model), Some(form)) = (self.kubernetes.clone(), self.kube_form.as_ref()) else {
            return;
        };
        let Some(version) = form.version_for(&label) else {
            return;
        };
        let mut wanted = model.read(cx).settings(cx);
        if wanted.version.as_deref() == Some(version.as_str()) {
            return;
        }
        wanted.version = Some(version.to_string());
        if model.read(cx).is_downgrade(&version) {
            kube_dialogs::downgrade(model, wanted, window, cx);
        } else {
            model.update(cx, |model, cx| model.save(wanted, cx));
        }
    }

    /// Checks the port, saves it, and applies the settings to the running engine.
    pub(super) fn apply_kubernetes(&mut self, cx: &mut Context<Self>) {
        let (Some(model), Some(form)) = (self.kubernetes.clone(), self.kube_form.as_mut()) else {
            return;
        };
        let text = form.port.read(cx).value().trim().to_string();
        let port = match text.parse::<u16>() {
            Ok(port) if port > 0 => port,
            _ => {
                form.error = Some(format!("{text:?} is not a port number.").into());
                cx.notify();
                return;
            }
        };
        form.error = None;
        model.update(cx, |model: &mut KubernetesModel, cx| {
            let wanted = KubernetesSettings {
                port,
                ..model.settings(cx)
            };
            model.save(wanted, cx);
            model.apply(cx);
        });
    }
}

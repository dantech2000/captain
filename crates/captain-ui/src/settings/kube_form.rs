//! The Kubernetes version picker. It needs a window, so the Settings page creates
//! it on its first render. The port lives in the settings file. See features 0024
//! and 0037.

use captain_core::kubernetes::{K3sVersion, KubernetesSettings, VersionList};
use gpui_kit::component::select::{SelectEvent, SelectState};
use gpui_kit::*;

use super::{SettingsView, kube_dialogs};

/// The version labels, such as `v1.36.4+k3s1 (stable)`.
pub type VersionSelect = SelectState<Vec<SharedString>>;

pub struct KubeForm {
    pub version: Entity<VersionSelect>,
    /// The versions behind the labels, in the same order.
    versions: Vec<K3sVersion>,
    labels: Vec<SharedString>,
}

impl KubeForm {
    pub fn new(window: &mut Window, cx: &mut App) -> Self {
        Self {
            version: cx.new(|cx| SelectState::new(Vec::new(), None, window, cx)),
            versions: Vec::new(),
            labels: Vec::new(),
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
        // A change from elsewhere, such as a snapshot restore, rebuilds the form so
        // it never shows, or saves back, old values.
        if self.kube_form_source.as_ref() != Some(settings) {
            self.kube_form = None;
        }
        if self.kube_form.is_none() && self.kubernetes.is_some() {
            self.kube_form_source = Some(settings.clone());
            let form = KubeForm::new(window, cx);
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
}

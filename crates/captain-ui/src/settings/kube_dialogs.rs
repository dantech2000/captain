//! The questions before Reset Kubernetes and before a downgrade.

use captain_core::kubernetes::KubernetesSettings;
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use crate::kubernetes::KubernetesModel;
use crate::widgets::danger_footer;

/// Asks before Reset Kubernetes, which deletes the workloads.
pub fn reset(model: Entity<KubernetesModel>, window: &mut Window, cx: &mut App) {
    window.open_alert_dialog(cx, move |alert, _, _| {
        let model = model.clone();
        alert
            .title("Reset Kubernetes?")
            .description(
                "This deletes all workloads and the cluster state, then starts an empty \
                 cluster. Images and containers you started with Docker stay.",
            )
            .footer(danger_footer(
                "Reset",
                "Delete the Kubernetes workloads and the cluster state. Images stay.",
            ))
            .on_ok(move |_, _, cx| {
                model.update(cx, |model, cx| model.reset(cx));
                true
            })
    });
}

/// Asks before going back to an older version, which needs a reset. On OK it saves
/// `wanted` and resets the cluster.
pub fn downgrade(
    model: Entity<KubernetesModel>,
    wanted: KubernetesSettings,
    window: &mut Window,
    cx: &mut App,
) {
    let version = wanted.version.clone().unwrap_or_default();
    window.open_alert_dialog(cx, move |alert, _, _| {
        let (model, wanted) = (model.clone(), wanted.clone());
        alert
            .title(format!("Go back to Kubernetes {version}?"))
            .description(
                "An older version needs an empty cluster, so Captain resets it. All \
                 workloads are deleted. Images stay.",
            )
            .footer(danger_footer(
                "Reset and downgrade",
                "Reset the cluster and install the older Kubernetes version.",
            ))
            .on_ok(move |_, _, cx| {
                let wanted = wanted.clone();
                model.update(cx, |model, cx| {
                    model.save(wanted, cx);
                    model.reset(cx);
                });
                true
            })
    });
}

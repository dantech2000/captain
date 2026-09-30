use captain_core::kubernetes::KubernetesStatus;

use crate::tray::dot::Light;
use crate::tray::entries::KubeEntry;
use crate::tray::menu_model::{TrayCommand, TrayItem, build};
use crate::tray::snapshot::TraySnapshot;
use crate::tray::test_support::{find, labels, light, snapshot};

#[test]
fn the_kubernetes_check_turns_it_on_or_off() {
    let kube = |enabled, status| TraySnapshot {
        kubernetes: Some(KubeEntry { enabled, status }),
        ..snapshot(Vec::new())
    };
    let menu = build(&kube(false, KubernetesStatus::Off));
    assert!(!labels(&menu).iter().any(|l| l.starts_with("Kubernetes:")));
    assert_eq!(
        find(&menu, "Kubernetes"),
        &TrayItem::Check {
            label: "Kubernetes".into(),
            command: TrayCommand::SetKubernetes(true),
            checked: false,
        }
    );

    let running = KubernetesStatus::Running {
        version: "v1.31.4+k3s1".into(),
    };
    let menu = build(&kube(true, running));
    let line = find(&menu, "Kubernetes: Running \u{b7} v1.31.4+k3s1");
    assert_eq!(light(line), Some(Light::Green));
    assert!(matches!(
        find(&menu, "Kubernetes"),
        TrayItem::Check {
            command: TrayCommand::SetKubernetes(false),
            checked: true,
            ..
        }
    ));
}

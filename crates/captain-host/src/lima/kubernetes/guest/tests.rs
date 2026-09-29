use captain_core::kubernetes::{KubernetesSettings, KubernetesStatus};

use super::{install_args, parse_status};

#[test]
fn install_passes_the_port_and_traefik_switch() {
    let settings = KubernetesSettings {
        port: 7443,
        traefik: false,
        ..KubernetesSettings::default()
    };
    let args = install_args(
        "captain",
        "/c/v1",
        "k3s-arm64",
        "img.tar.zst",
        &settings,
        "v1",
    );
    assert_eq!(
        &args[..6],
        ["shell", "--workdir", "/", "captain", "sudo", "-n"]
    );
    assert_eq!(
        args[args.len() - 6..],
        ["/c/v1", "k3s-arm64", "img.tar.zst", "v1", "7443", "0"]
    );
}

#[test]
fn reads_the_unit_state_and_version() {
    assert_eq!(
        parse_status("active\nv1.36.4+k3s1\n"),
        KubernetesStatus::Running {
            version: "v1.36.4+k3s1".into()
        }
    );
    assert_eq!(parse_status("activating\n"), KubernetesStatus::Starting);
    assert!(matches!(
        parse_status("failed\n"),
        KubernetesStatus::Failed(_)
    ));
    assert_eq!(parse_status("inactive\nv1\n"), KubernetesStatus::Off);
}

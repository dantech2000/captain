use captain_core::kubernetes::KubernetesSettings;

use super::apply_enable;

#[test]
fn enable_keeps_the_fields_it_does_not_name() {
    let mut saved = KubernetesSettings {
        version: Some("v1.36.4+k3s1".into()),
        port: 7443,
        traefik: true,
        ..KubernetesSettings::default()
    };
    apply_enable(&mut saved, None, None, None, Some(false));
    assert!(saved.enabled);
    assert_eq!(saved.version.as_deref(), Some("v1.36.4+k3s1"));
    assert_eq!(saved.port, 7443);
    assert!(!saved.traefik);
}

use captain_core::settings::Settings;

use super::option_entries;

#[test]
fn nested_settings_get_dotted_keys_and_search_finds_them() {
    let entries = option_entries(&Settings::default());
    let port = entries
        .iter()
        .find(|entry| entry.key == "kubernetes.port")
        .expect("kubernetes.port is listed");
    assert!(port.matches("KUBERNETES"));
    assert!(!port.description.is_empty());
    assert!(!port.matches("traefik"));
    assert!(entries.iter().all(|entry| entry.key != "version"));
}

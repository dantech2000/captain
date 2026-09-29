use super::{Backend, ExtensionMetadata};

const FULL: &str = r#"{
  "icon": "docker.svg",
  "ui": {"dashboard-tab": {"title": "My Ext", "root": "/ui", "src": "index.html"}},
  "vm": {"image": "${DESKTOP_PLUGIN_IMAGE}", "exposes": {"socket": "backend.sock"}},
  "host": {"binaries": [
    {"darwin": [{"path": "/darwin/tool"}], "linux": [{"path": "/linux/tool"}]},
    {"darwin": [{"path": "/darwin/helper"}]}
  ]}
}"#;

#[test]
fn parses_every_section() {
    let metadata = ExtensionMetadata::parse(FULL).unwrap();
    let tab = metadata.dashboard_tab().unwrap();
    assert_eq!((tab.root.as_str(), tab.src.as_str()), ("/ui", "index.html"));
    assert_eq!(
        metadata.backend("acme/ext:1.0"),
        Some(Backend::Image("acme/ext:1.0".into()))
    );
    assert_eq!(metadata.backend_socket(), Some("backend.sock"));
    assert_eq!(
        metadata.host_binaries("darwin"),
        ["/darwin/tool", "/darwin/helper"]
    );
    assert!(metadata.host_binaries("windows").is_empty());
}

#[test]
fn a_ui_only_extension_has_no_backend() {
    let json = r#"{"ui": {"dashboard-tab": {"title": "T", "root": "/ui", "src": "index.html"}}}"#;
    let metadata = ExtensionMetadata::parse(json).unwrap();
    assert_eq!(metadata.backend("x"), None);
    assert!(metadata.host_binaries("darwin").is_empty());
}

#[test]
fn a_compose_backend_keeps_its_path() {
    let json = r#"{"vm": {"composefile": "docker-compose.yaml"}}"#;
    let metadata = ExtensionMetadata::parse(json).unwrap();
    assert_eq!(
        metadata.backend("x"),
        Some(Backend::Compose("docker-compose.yaml".into()))
    );
}

#[test]
fn rejects_bad_json_and_an_empty_vm() {
    assert!(ExtensionMetadata::parse("not json").is_err());
    assert!(ExtensionMetadata::parse(r#"{"vm": {}}"#).is_err());
}

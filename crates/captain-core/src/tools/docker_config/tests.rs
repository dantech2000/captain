use std::path::Path;

use serde_json::{Value, json};

use super::docker_config;

const BUNDLED: &str = "/Applications/Captain.app/Contents/Resources/cli-plugins";
const USER_PLUGINS: &str = "/Users/dan/.docker/cli-plugins";

fn build(user: Option<&str>) -> Value {
    let text = docker_config(user, Path::new(BUNDLED), Path::new(USER_PLUGINS));
    serde_json::from_str(&text).expect("valid JSON")
}

#[test]
fn without_a_user_config_only_the_plugin_folders_are_set() {
    assert_eq!(
        build(None),
        json!({ "cliPluginsExtraDirs": [BUNDLED, USER_PLUGINS] })
    );
}

#[test]
fn keeps_credentials_and_puts_bundled_plugins_first() {
    let user = r#"{
        "auths": { "ghcr.io": {} },
        "credsStore": "osxkeychain",
        "currentContext": "rancher-desktop",
        "cliPluginsExtraDirs": ["/opt/plugins"]
    }"#;
    assert_eq!(
        build(Some(user)),
        json!({
            "auths": { "ghcr.io": {} },
            "credsStore": "osxkeychain",
            "cliPluginsExtraDirs": [BUNDLED, "/opt/plugins", USER_PLUGINS]
        })
    );
}

#[test]
fn a_broken_user_config_counts_as_empty() {
    assert_eq!(build(Some("{ not json")), build(None));
}

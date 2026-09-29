use std::collections::HashMap;

use captain_core::model::ProjectAction;

use super::{label_override, list_args, stdin_file_args};

#[test]
fn override_on_stdin_labels_every_listed_service() {
    let args: Vec<String> = ["compose", "-p", "shop", "-f", "/a.yaml", "up", "-d"]
        .map(String::from)
        .into();
    assert_eq!(
        stdin_file_args(&args, ProjectAction::Up),
        [
            "compose", "-p", "shop", "-f", "/a.yaml", "-f", "-", "up", "-d"
        ]
    );
    assert_eq!(
        list_args(&args, ProjectAction::Up),
        [
            "compose",
            "-p",
            "shop",
            "-f",
            "/a.yaml",
            "--profile",
            "*",
            "config",
            "--services"
        ]
    );
    let labels = HashMap::from([("dev.captain.migrated-from".into(), "ID:1".into())]);
    let json: serde_json::Value =
        serde_json::from_str(&label_override(&["web", "", "db"], &labels)).expect("json");
    assert_eq!(
        json,
        serde_json::json!({ "services": {
            "web": { "labels": { "dev.captain.migrated-from": "ID:1" } },
            "db": { "labels": { "dev.captain.migrated-from": "ID:1" } },
        }})
    );
}

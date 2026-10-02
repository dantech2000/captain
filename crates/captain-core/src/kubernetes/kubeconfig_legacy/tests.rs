use serde_json::{Value, json};

use super::remove_legacy;
use crate::kubernetes::kubeconfig::{CONTEXT, contexts, current_context};

/// A config with one `captain` cluster, user, and context, as Captain wrote them
/// before the rename, next to the user's `prod` context.
fn old(server: &str, ca: &str) -> Value {
    json!({
        "clusters": [
            {"name": "prod", "cluster": {"server": "https://prod.example.com"}},
            {"name": "captain", "cluster": {"server": server, "certificate-authority-data": ca}},
        ],
        "users": [{"name": "captain", "user": {"token": "t"}}],
        "contexts": [
            {"name": "prod", "context": {"cluster": "prod", "user": "prod"}},
            {"name": "captain", "context": {"cluster": "captain", "user": "captain"}},
        ],
        "current-context": "captain",
    })
}

#[test]
fn removes_captains_old_entries_and_moves_the_current_context() {
    let cas = ["Q0E=".to_string()];
    let migrated = remove_legacy(&old("https://127.0.0.1:6443", "Q0E="), &cas);
    assert_eq!(contexts(&migrated), ["prod"]);
    assert_eq!(migrated["clusters"].as_array().unwrap().len(), 1);
    assert_eq!(migrated["users"].as_array().unwrap().len(), 0);
    assert_eq!(current_context(&migrated).as_deref(), Some(CONTEXT));
}

#[test]
fn keeps_a_captain_context_that_is_not_captains() {
    let cas = ["Q0E=".to_string()];
    for config in [
        old("https://127.0.0.1:6443", "T1RIRVI="),
        old("https://captain.example.com", "Q0E="),
    ] {
        assert_eq!(remove_legacy(&config, &cas), config);
    }
    let mut other_user = old("https://127.0.0.1:6443", "Q0E=");
    other_user["contexts"][1]["context"]["user"] = "me".into();
    assert_eq!(remove_legacy(&other_user, &cas), other_user);
    let unknown = old("https://127.0.0.1:6443", "Q0E=");
    assert_eq!(remove_legacy(&unknown, &[]), unknown);
}

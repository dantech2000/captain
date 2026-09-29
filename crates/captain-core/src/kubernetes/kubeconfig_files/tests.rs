use std::path::{Path, PathBuf};

use super::{install_captain, kubeconfig_paths, load_contexts, use_context};
use crate::kubernetes::kubeconfig::{CONTEXT, current_context};
use crate::kubernetes::read_config;
use serde_json::json;

fn folder(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("captain-kubeconfig-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn captain() -> serde_json::Value {
    json!({
        "clusters": [{"name": CONTEXT, "cluster": {"server": "https://127.0.0.1:6443"}}],
        "users": [{"name": CONTEXT, "user": {"token": "t"}}],
        "contexts": [{"name": CONTEXT, "context": {"cluster": CONTEXT, "user": CONTEXT}}],
    })
}

#[test]
fn reads_the_kubeconfig_list_or_the_default() {
    let home = Path::new("/home/me");
    assert_eq!(
        kubeconfig_paths(None, home),
        [home.join(".kube").join("config")]
    );
    let joined = std::env::join_paths(["/a/one", "/b/two"]).unwrap();
    assert_eq!(
        kubeconfig_paths(Some(&joined), home),
        [PathBuf::from("/a/one"), PathBuf::from("/b/two")]
    );
}

#[test]
fn installs_into_the_file_with_captain_and_backs_it_up() {
    let dir = folder("install");
    let (first, second) = (dir.join("first"), dir.join("second"));
    std::fs::write(&first, "current-context: prod\ncontexts:\n- name: prod\n").unwrap();
    std::fs::write(&second, "contexts:\n- name: captain\n").unwrap();
    let paths = [first.clone(), second.clone()];

    assert_eq!(install_captain(&paths, &captain()).unwrap(), second);
    assert!(dir.join("second.captain-backup").exists());
    let contexts = load_contexts(&paths);
    assert_eq!(contexts.names, ["prod", "captain"]);
    assert_eq!(contexts.current.as_deref(), Some("prod"));

    use_context(&paths, CONTEXT).unwrap();
    let first = read_config(&first).unwrap();
    assert_eq!(current_context(&first).as_deref(), Some(CONTEXT));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn installs_into_the_first_file_when_none_has_captain() {
    let dir = folder("first");
    let paths = [dir.join("new"), dir.join("other")];
    assert_eq!(install_captain(&paths, &captain()).unwrap(), paths[0]);
    assert_eq!(load_contexts(&paths).current.as_deref(), Some(CONTEXT));
    std::fs::remove_dir_all(&dir).ok();
}

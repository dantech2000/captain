use std::path::{Path, PathBuf};

use super::{
    install_captain, kubeconfig_paths, load_contexts, read_config, uninstall_captain, use_context,
    write_config,
};
use crate::kubernetes::kubeconfig::{CONTEXT, contexts, current_context, merge};
use crate::kubernetes::kubeconfig_legacy::LEGACY_CONTEXT;
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
    std::fs::write(&second, "contexts:\n- name: captain-desktop\n").unwrap();
    let paths = [first.clone(), second.clone()];

    assert_eq!(install_captain(&paths, &captain()).unwrap(), second);
    assert!(dir.join("second.captain-backup").exists());
    let contexts = load_contexts(&paths);
    assert_eq!(contexts.names, ["prod", CONTEXT]);
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

#[test]
fn keeps_the_current_context_that_a_later_file_sets() {
    let dir = folder("later");
    let (first, second) = (dir.join("first"), dir.join("second"));
    std::fs::write(&first, "contexts:\n- name: dev\n").unwrap();
    std::fs::write(&second, "current-context: production\n").unwrap();
    let paths = [first.clone(), second];
    assert_eq!(install_captain(&paths, &captain()).unwrap(), first);
    assert_eq!(load_contexts(&paths).current.as_deref(), Some("production"));
    assert_eq!(load_contexts(&paths).names, ["dev", CONTEXT]);
    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(unix)]
#[test]
fn a_symlinked_kubeconfig_stays_a_link() {
    use std::os::unix::fs::PermissionsExt;
    let dir = folder("symlink");
    std::fs::create_dir_all(dir.join("dotfiles")).unwrap();
    let link = dir.join("config");
    std::os::unix::fs::symlink(dir.join("dotfiles/config"), &link).unwrap();
    install_captain(std::slice::from_ref(&link), &captain()).unwrap();
    assert!(link.symlink_metadata().unwrap().file_type().is_symlink());
    let target = std::fs::metadata(dir.join("dotfiles/config")).unwrap();
    assert_eq!(target.permissions().mode() & 0o777, 0o600);
    assert!(current_context(&read_config(&link).unwrap()).is_some());
    std::fs::remove_dir_all(&dir).ok();
}

/// Captain's entries from before the rename, with the same authority as
/// [`captain_with_ca`], as the current context.
const OLD_CAPTAIN: &str = "current-context: captain
clusters:
- name: captain
  cluster:
    server: https://127.0.0.1:6443
    certificate-authority-data: Q0E=
users:
- name: captain
  user:
    token: old
contexts:
- name: captain
  context:
    cluster: captain
    user: captain
";

/// A user's own `captain` context for another cluster.
const USER_CAPTAIN: &str = "clusters:
- name: captain
  cluster:
    server: https://captain.example.com
contexts:
- name: captain
  context:
    cluster: captain
    user: me
";

fn captain_with_ca() -> serde_json::Value {
    let mut captain = captain();
    captain["clusters"][0]["cluster"]["certificate-authority-data"] = "Q0E=".into();
    captain
}

#[test]
fn install_replaces_captains_old_entries_and_keeps_a_users_captain() {
    let dir = folder("legacy");
    let (first, second) = (dir.join("first"), dir.join("second"));
    std::fs::write(&first, USER_CAPTAIN).unwrap();
    std::fs::write(&second, OLD_CAPTAIN).unwrap();
    let paths = [first.clone(), second.clone()];

    assert_eq!(install_captain(&paths, &captain_with_ca()).unwrap(), second);
    assert_eq!(std::fs::read_to_string(&first).unwrap(), USER_CAPTAIN);
    let backup = std::fs::read_to_string(dir.join("second.captain-backup")).unwrap();
    assert_eq!(backup, OLD_CAPTAIN);
    let second = read_config(&second).unwrap();
    assert_eq!(contexts(&second), [CONTEXT]);
    assert_eq!(current_context(&second).as_deref(), Some(CONTEXT));
    assert_eq!(load_contexts(&paths).names, [LEGACY_CONTEXT, CONTEXT]);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn uninstall_removes_both_names_when_they_are_captains() {
    let dir = folder("uninstall");
    let (first, second) = (dir.join("first"), dir.join("second"));
    std::fs::write(&first, OLD_CAPTAIN).unwrap();
    std::fs::write(&second, USER_CAPTAIN).unwrap();
    let paths = [first.clone(), second.clone()];
    let mut both = read_config(&first).unwrap();
    both = merge(&both, &captain_with_ca());
    write_config(&first, &both).unwrap();

    uninstall_captain(&paths).unwrap();
    let first = read_config(&first).unwrap();
    assert!(contexts(&first).is_empty());
    assert_eq!(current_context(&first), None);
    assert_eq!(std::fs::read_to_string(&second).unwrap(), USER_CAPTAIN);
    std::fs::remove_dir_all(&dir).ok();
}

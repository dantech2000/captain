use captain_core::extension::{ExtensionCandidate, ExtensionPaths, InstalledExtension};

use super::{check_absent, delete_folders, list};

fn write(paths: &ExtensionPaths, id: &str, engine: &str) {
    write_in(paths, id, id, engine);
}

fn write_in(paths: &ExtensionPaths, folder: &str, id: &str, engine: &str) {
    let mut extension = InstalledExtension::new(
        ExtensionCandidate {
            id: id.into(),
            image: format!("acme/{id}"),
            image_id: String::new(),
            labels: Default::default(),
            metadata: Default::default(),
        },
        0,
    );
    extension.engine = engine.into();
    std::fs::create_dir_all(paths.dir(folder)).unwrap();
    std::fs::write(paths.manifest(folder), extension.to_json()).unwrap();
}

#[test]
fn the_list_holds_the_extensions_of_this_engine_and_older_installs_in_their_own_folders() {
    let root = std::env::temp_dir().join(format!("captain-ext-list-{}", std::process::id()));
    let paths = ExtensionPaths::new(root.clone());
    write(&paths, "a", "unix:///a.sock");
    write(&paths, "b", "unix:///b.sock");
    write(&paths, "old", "");
    write_in(&paths, "evil", "../..", "");
    let ids: Vec<String> = list(&paths, "unix:///a.sock")
        .unwrap()
        .into_iter()
        .map(|extension| extension.id)
        .collect();
    std::fs::remove_dir_all(&root).ok();
    assert_eq!(ids, ["a", "old"]);
}

#[test]
fn install_refuses_an_unreadable_manifest_or_an_unfinished_update_until_remove() {
    let root = std::env::temp_dir().join(format!("captain-ext-absent-{}", std::process::id()));
    let paths = ExtensionPaths::new(root.clone());
    std::fs::create_dir_all(paths.dir("a")).unwrap();
    std::fs::write(paths.manifest("a"), "{ not json").unwrap();
    std::fs::create_dir_all(paths.backup_dir("b")).unwrap();
    std::fs::create_dir_all(paths.dir("c")).unwrap();
    let refused = [
        check_absent(&paths, "a").is_err(),
        check_absent(&paths, "b").is_err(),
    ];
    let leftover_ok = check_absent(&paths, "c").is_ok();
    delete_folders(&paths, "b").unwrap();
    let after_remove = check_absent(&paths, "b").is_ok();
    std::fs::remove_dir_all(&root).ok();
    assert_eq!(refused, [true, true]);
    assert!(leftover_ok && after_remove);
}

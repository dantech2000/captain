use captain_core::extension::{ExtensionCandidate, ExtensionPaths, InstalledExtension};

use super::list;

fn write(paths: &ExtensionPaths, id: &str, engine: &str) {
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
    std::fs::create_dir_all(paths.dir(id)).unwrap();
    std::fs::write(paths.manifest(id), extension.to_json()).unwrap();
}

#[test]
fn the_list_holds_the_extensions_of_this_engine_and_older_installs() {
    let root = std::env::temp_dir().join(format!("captain-ext-list-{}", std::process::id()));
    let paths = ExtensionPaths::new(root.clone());
    write(&paths, "a", "unix:///a.sock");
    write(&paths, "b", "unix:///b.sock");
    write(&paths, "old", "");
    let ids: Vec<String> = list(&paths, "unix:///a.sock")
        .unwrap()
        .into_iter()
        .map(|extension| extension.id)
        .collect();
    std::fs::remove_dir_all(&root).ok();
    assert_eq!(ids, ["a", "old"]);
}

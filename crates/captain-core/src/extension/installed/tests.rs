use super::{ExtensionCandidate, InstalledExtension};

#[test]
fn the_pinned_image_is_the_id_and_the_reference_for_older_installs() {
    let candidate = |image_id: &str| ExtensionCandidate {
        id: "acme-ext-12345678".into(),
        image: "acme/ext:1".into(),
        image_id: image_id.into(),
        labels: Default::default(),
        metadata: Default::default(),
    };
    let pinned = InstalledExtension::new(candidate("sha256:abc"), 0);
    assert_eq!(pinned.pinned_image(), "sha256:abc");
    let older = InstalledExtension::new(candidate(""), 0);
    assert_eq!(older.pinned_image(), "acme/ext:1");
}

#[cfg(unix)]
#[test]
fn an_extension_runs_on_its_engine_through_a_symlinked_socket_path() {
    let root = std::env::temp_dir().join(format!("captain-engine-key-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let socket = root.join("docker.sock");
    std::fs::write(&socket, "").unwrap();
    let link = root.join("link.sock");
    std::os::unix::fs::symlink(&socket, &link).unwrap();
    let mut extension = InstalledExtension::new(
        ExtensionCandidate {
            id: "a".into(),
            image: "acme/a".into(),
            image_id: String::new(),
            labels: Default::default(),
            metadata: Default::default(),
        },
        0,
    );
    extension.engine = format!("unix://{}", socket.display());
    let through_link = extension.runs_on(&format!("unix://{}", link.display()));
    let elsewhere = extension.runs_on("unix:///nowhere/docker.sock");
    std::fs::remove_dir_all(&root).ok();
    assert!(through_link && !elsewhere);
}

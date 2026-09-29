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

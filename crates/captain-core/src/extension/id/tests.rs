use super::{data_store_id, extension_id, project_name};

/// The ID without its hash.
fn slug(reference: &str) -> Option<String> {
    let id = extension_id(reference)?;
    let (slug, hash) = id.rsplit_once('-')?;
    assert!(hash.len() == 8 && hash.bytes().all(|b| b.is_ascii_hexdigit()));
    Some(slug.to_string())
}

#[test]
fn the_id_is_the_repository_as_a_host_name() {
    assert_eq!(
        slug("docker/disk-usage-extension:0.2.9").as_deref(),
        Some("docker-disk-usage-extension")
    );
    assert_eq!(
        slug("ghcr.io/Acme/My_Ext@sha256:abc").as_deref(),
        Some("ghcr-io-acme-my-ext")
    );
    assert_eq!(
        slug("localhost:5000/ext").as_deref(),
        Some("localhost-5000-ext")
    );
    assert_eq!(extension_id("two words"), None);
}

#[test]
fn repositories_with_the_same_slug_get_different_ids() {
    assert_eq!(
        extension_id("acme/foo-bar:1"),
        extension_id("acme/foo-bar:2")
    );
    assert_ne!(extension_id("acme/foo-bar"), extension_id("acme/foo_bar"));
}

#[test]
fn the_project_and_the_data_store_follow_the_id() {
    assert_eq!(project_name("acme-ext"), "captain-ext-acme-ext");
    assert_eq!(data_store_id("acme-ext"), data_store_id("acme-ext"));
    assert_ne!(data_store_id("acme-ext"), data_store_id("acme-ext2"));
}

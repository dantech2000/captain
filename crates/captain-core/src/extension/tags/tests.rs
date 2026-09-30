use super::{choose_tag, untagged_repository};

fn choose(tags: &[&str]) -> String {
    choose_tag(&tags.iter().map(|t| t.to_string()).collect::<Vec<_>>())
}

#[test]
fn the_newest_release_wins_over_junk_tags_and_pre_releases() {
    assert_eq!(
        choose(&[
            "0.0.1-alpha.8",
            "0.2.9",
            "0.10.0",
            "0.2.10",
            "test-1",
            "main",
            "pr-214"
        ]),
        "0.10.0"
    );
    assert_eq!(choose(&["v1.2.0", "1.10.0-rc.1", "latest"]), "v1.2.0");
    assert_eq!(
        choose(&["1.0.0-rc.1", "1.0.0-rc.2", "nightly"]),
        "1.0.0-rc.2"
    );
    assert_eq!(choose(&["latest", "main"]), "latest");
    assert_eq!(choose(&[]), "latest");
}

#[test]
fn only_a_reference_without_tag_or_digest_needs_a_tag() {
    assert_eq!(
        untagged_repository("docker/disk-usage-extension").as_deref(),
        Some("docker/disk-usage-extension")
    );
    assert_eq!(
        untagged_repository("localhost:5000/ext").as_deref(),
        Some("localhost:5000/ext")
    );
    assert_eq!(
        untagged_repository("docker/disk-usage-extension:0.2.9"),
        None
    );
    assert_eq!(untagged_repository("ext@sha256:abc"), None);
}

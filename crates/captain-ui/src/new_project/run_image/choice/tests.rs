use super::split_reference;

#[test]
fn references_split_into_repository_and_tag() {
    let split = |text| split_reference(text);
    assert_eq!(split("postgres:18"), ("postgres".into(), "18".into(), None));
    assert_eq!(
        split("localhost:5000/api"),
        ("localhost:5000/api".into(), String::new(), None)
    );
    assert_eq!(
        split("ghcr.io/acme/api:1.2@sha256:abc"),
        (
            "ghcr.io/acme/api".into(),
            "1.2".into(),
            Some("sha256:abc".into())
        )
    );
    assert_eq!(
        split("registry/app@sha256:abc"),
        (
            "registry/app".into(),
            String::new(),
            Some("sha256:abc".into())
        )
    );
    assert_eq!(
        split("sha256:4a3f"),
        ("sha256:4a3f".into(), String::new(), None)
    );
}

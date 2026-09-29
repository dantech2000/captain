use super::ImageReference;

fn parse(input: &str) -> Option<(String, String)> {
    ImageReference::parse(input).map(|r| (r.name, r.tag))
}

fn pair(name: &str, tag: &str) -> Option<(String, String)> {
    Some((name.to_string(), tag.to_string()))
}

#[test]
fn a_missing_tag_means_latest() {
    assert_eq!(parse("busybox"), pair("busybox", "latest"));
    assert_eq!(parse("  busybox  "), pair("busybox", "latest"));
}

#[test]
fn an_explicit_tag_is_kept() {
    assert_eq!(parse("nginx:1.27-alpine"), pair("nginx", "1.27-alpine"));
    assert_eq!(parse("ghcr.io/org/app:v2"), pair("ghcr.io/org/app", "v2"));
}

#[test]
fn a_registry_port_is_not_a_tag() {
    assert_eq!(
        parse("localhost:5000/app"),
        pair("localhost:5000/app", "latest")
    );
    assert_eq!(
        parse("localhost:5000/app:dev"),
        pair("localhost:5000/app", "dev")
    );
}

#[test]
fn a_digest_has_no_tag() {
    assert_eq!(parse("alpine@sha256:abc"), pair("alpine@sha256:abc", ""));
}

#[test]
fn bad_input_is_rejected() {
    assert_eq!(parse(""), None);
    assert_eq!(parse("   "), None);
    assert_eq!(parse("two words"), None);
    assert_eq!(parse("nginx:"), None);
    assert_eq!(parse(":tag"), None);
}

#[test]
fn display_joins_name_and_tag() {
    let busybox = ImageReference::parse("busybox").unwrap();
    assert_eq!(busybox.to_string(), "busybox:latest");
    let digest = ImageReference::parse("alpine@sha256:abc").unwrap();
    assert_eq!(digest.to_string(), "alpine@sha256:abc");
}

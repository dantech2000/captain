use super::ImageReference;

fn parse(input: &str) -> Option<(String, String)> {
    ImageReference::parse(input).map(|r| (r.name, r.tag))
}

#[test]
fn references_split_into_name_and_tag() {
    let cases = [
        // A missing tag means latest.
        ("busybox", Some(("busybox", "latest"))),
        ("  busybox  ", Some(("busybox", "latest"))),
        ("nginx:1.27-alpine", Some(("nginx", "1.27-alpine"))),
        ("ghcr.io/org/app:v2", Some(("ghcr.io/org/app", "v2"))),
        // A registry port is not a tag.
        ("localhost:5000/app", Some(("localhost:5000/app", "latest"))),
        (
            "localhost:5000/app:dev",
            Some(("localhost:5000/app", "dev")),
        ),
        // A digest has no tag.
        ("alpine@sha256:abc", Some(("alpine@sha256:abc", ""))),
        ("", None),
        ("   ", None),
        ("two words", None),
        ("nginx:", None),
        (":tag", None),
    ];
    for (input, expected) in cases {
        let expected = expected.map(|(name, tag)| (name.to_string(), tag.to_string()));
        assert_eq!(parse(input), expected, "{input:?}");
    }
    let busybox = ImageReference::parse("busybox").unwrap();
    assert_eq!(busybox.to_string(), "busybox:latest");
    let digest = ImageReference::parse("alpine@sha256:abc").unwrap();
    assert_eq!(digest.to_string(), "alpine@sha256:abc");
}

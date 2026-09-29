use super::parse_version;

#[test]
fn reads_release_versions() {
    assert_eq!(parse_version("limactl version 2.2.0\n"), Some((2, 2, 0)));
    assert_eq!(parse_version("limactl version v1.0.7"), Some((1, 0, 7)));
    assert_eq!(
        parse_version("limactl version 2.3.0-12-gabc"),
        Some((2, 3, 0))
    );
    assert_eq!(parse_version("limactl version 2.2"), Some((2, 2, 0)));
}

#[test]
fn development_builds_are_unknown() {
    assert_eq!(parse_version("limactl version HEAD-abcdef"), None);
    assert_eq!(parse_version(""), None);
}

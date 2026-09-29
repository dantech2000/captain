use super::{check_version, parse_version};

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

#[test]
fn needs_2_2_0() {
    assert_eq!(check_version("limactl version 2.2.0"), Ok(()));
    assert_eq!(check_version("limactl version 3.0.1"), Ok(()));
    assert_eq!(check_version("limactl version HEAD-abc"), Ok(()));
    let error = check_version("limactl version 2.1.9").unwrap_err();
    assert!(
        error.starts_with("Lima 2.2.0 or newer is required"),
        "{error}"
    );
}

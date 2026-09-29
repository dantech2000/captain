use super::check_version;

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

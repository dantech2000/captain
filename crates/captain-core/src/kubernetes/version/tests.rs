use super::K3sVersion;

fn v(text: &str) -> K3sVersion {
    text.parse().unwrap()
}

#[test]
fn orders_by_number_then_release_then_build() {
    assert!(v("v1.36.4+k3s1") > v("v1.35.9+k3s1"));
    assert!(v("v1.36.10+k3s1") > v("v1.36.9+k3s1"));
    assert!(v("v1.37.0+k3s1") > v("v1.37.0-rc5+k3s1"));
    assert!(v("v1.33.13+k3s2") > v("v1.33.13+k3s1"));
}

#[test]
fn only_releases_are_stable() {
    assert!(v("v1.36.4+k3s1").is_stable());
    assert!(!v("v1.37.1-rc2+k3s1").is_stable());
    assert_eq!(v("v1.36.4+k3s1").minor_channel(), "v1.36");
}

#[test]
fn rejects_other_tags() {
    for text in [
        "1.36.4+k3s1",
        "v1.36+k3s1",
        "v1.36.4",
        "latest",
        "v1.36.4-rc/../../x+k3s1",
        "v1.36.4-+k3s1",
        "v1.36.4+k3s+1",
    ] {
        assert!(text.parse::<K3sVersion>().is_err(), "{text}");
    }
}

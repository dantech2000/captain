use super::{VersionList, parse_channels, parse_releases};

const CHANNELS: &str = r#"{"data": [
  {"name": "stable", "latest": "v1.36.4+k3s1"},
  {"name": "latest", "latest": "v1.37.0+k3s1"},
  {"name": "testing", "latest": "v1.18.2-rc3+k3s1"},
  {"name": "v1.36", "latest": "v1.36.4+k3s1"},
  {"name": "v1.36-testing"},
  {"name": "v1.16", "latest": "v1.16.15+k3s1"}
]}"#;

const RELEASES: &str = r#"[
  {"tag_name": "v1.37.1-rc2+k3s1", "prerelease": true, "draft": false},
  {"tag_name": "v1.37.0+k3s1", "prerelease": false, "draft": false},
  {"tag_name": "v1.36.4+k3s1", "prerelease": false, "draft": false},
  {"tag_name": "v1.36.5+k3s1", "prerelease": false, "draft": true},
  {"tag_name": "v1.28.6+k3s1", "prerelease": false, "draft": false}
]"#;

fn list() -> VersionList {
    VersionList::new(
        parse_releases(RELEASES).unwrap(),
        parse_channels(CHANNELS).unwrap(),
    )
}

#[test]
fn keeps_stable_releases_above_the_floor_and_skips_testing_channels() {
    let names: Vec<String> = list().versions.iter().map(ToString::to_string).collect();
    assert_eq!(names, ["v1.37.0+k3s1", "v1.36.4+k3s1"]);
    let channels = parse_channels(CHANNELS).unwrap();
    let names: Vec<&str> = channels.keys().map(String::as_str).collect();
    assert_eq!(names, ["latest", "stable", "v1.16", "v1.36"]);
}

#[test]
fn labels_a_version_with_the_channels_it_leads() {
    let list = list();
    assert_eq!(list.stable().unwrap().as_str(), "v1.36.4+k3s1");
    assert_eq!(
        list.label(&list.versions[1]),
        "v1.36.4+k3s1 (stable, v1.36)"
    );
    assert_eq!(list.label(&list.versions[0]), "v1.37.0+k3s1 (latest)");
}

#[test]
fn includes_downloaded_versions_in_order() {
    let mut list = list();
    list.include(["v1.36.10+k3s1".parse().unwrap()]);
    assert_eq!(list.versions[1].as_str(), "v1.36.10+k3s1");
}

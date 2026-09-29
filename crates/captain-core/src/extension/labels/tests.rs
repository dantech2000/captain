use std::collections::HashMap;

use super::ExtensionLabels;

fn labels(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn reads_the_title_and_the_publisher() {
    let read = ExtensionLabels::from_image(&labels(&[
        ("com.docker.desktop.extension.api.version", ">= 0.3.0"),
        ("org.opencontainers.image.title", "Disk Usage"),
        ("org.opencontainers.image.vendor", "Docker Inc."),
    ]))
    .unwrap();
    assert_eq!(read.title, "Disk Usage");
    assert_eq!(read.publisher(), "Docker Inc.");
    assert_eq!(read.api_version, ">= 0.3.0");
}

#[test]
fn an_image_without_the_api_version_is_not_an_extension() {
    let error =
        ExtensionLabels::from_image(&labels(&[("org.opencontainers.image.title", "nginx")]));
    assert!(error.unwrap_err().contains("not a Docker extension"));
}

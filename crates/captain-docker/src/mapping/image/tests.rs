use std::collections::HashMap;

use bollard::errors::Error;
use bollard::models::{ContainerSummary, CreateImageInfo, ImageSummary, ProgressDetail};
use captain_core::EngineError;

use super::{image, image_usage, needs_usage, pull_error, pull_progress};

fn summary(id: &str, tags: &[&str], containers: i64) -> ImageSummary {
    ImageSummary {
        id: id.into(),
        repo_tags: tags.iter().map(|t| t.to_string()).collect(),
        size: 1024,
        created: 1_700_000_000,
        containers,
        ..ImageSummary::default()
    }
}

#[test]
fn maps_a_tagged_image() {
    let image = image(summary("sha256:a", &["nginx:1.27"], 2), &HashMap::new());
    assert_eq!(image.id, "sha256:a");
    assert_eq!(image.repo_tags, ["nginx:1.27"]);
    assert_eq!(image.size, 1024);
    assert_eq!(image.created, 1_700_000_000);
    assert_eq!(image.containers, 2);
    assert!(!image.dangling);
}

#[test]
fn none_tags_mean_dangling() {
    let image = image(summary("sha256:b", &["<none>:<none>"], 0), &HashMap::new());
    assert!(image.repo_tags.is_empty());
    assert!(image.dangling);
}

#[test]
fn unknown_counts_come_from_the_container_list() {
    let containers = vec![
        ContainerSummary {
            image_id: Some("sha256:a".into()),
            ..ContainerSummary::default()
        },
        ContainerSummary {
            image_id: Some("sha256:a".into()),
            ..ContainerSummary::default()
        },
        ContainerSummary::default(),
    ];
    let usage = image_usage(&containers);
    assert_eq!(usage.get("sha256:a"), Some(&2));

    let summaries = vec![summary("sha256:a", &["a:1"], -1)];
    assert!(needs_usage(&summaries));
    let mapped = image(summaries[0].clone(), &usage);
    assert_eq!(mapped.containers, 2);
    assert_eq!(image(summary("sha256:z", &[], -1), &usage).containers, 0);
}

#[test]
fn maps_pull_messages() {
    let progress = pull_progress(CreateImageInfo {
        id: Some("a1b2".into()),
        status: Some("Downloading".into()),
        progress_detail: Some(ProgressDetail {
            current: Some(10),
            total: Some(40),
        }),
        ..CreateImageInfo::default()
    });
    assert_eq!(progress.layer.as_deref(), Some("a1b2"));
    assert_eq!(progress.status, "Downloading");
    assert_eq!((progress.current, progress.total), (Some(10), Some(40)));

    let status = pull_progress(CreateImageInfo {
        status: Some("Digest: sha256:abc".into()),
        ..CreateImageInfo::default()
    });
    assert_eq!(status.layer, None);
    assert_eq!(status.current, None);
}

#[test]
fn stream_errors_are_api_errors() {
    let error = pull_error(Error::DockerStreamError {
        error: "manifest unknown".into(),
    });
    assert_eq!(error, EngineError::Api("manifest unknown".into()));
}

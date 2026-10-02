use std::collections::HashMap;

use bollard::errors::Error;
use bollard::models::{ImageConfig, ImageHistoryResponseItem, ImageInspect};

use super::{image_detail, image_history, image_layer};

fn nginx() -> ImageInspect {
    ImageInspect {
        id: Some("sha256:abc".into()),
        repo_tags: Some(vec!["nginx:1.27".into()]),
        repo_digests: Some(vec!["nginx@sha256:def".into()]),
        created: Some("2024-05-01T12:30:45Z".into()),
        architecture: Some("arm64".into()),
        variant: Some("v8".into()),
        os: Some("linux".into()),
        size: Some(1024),
        config: Some(ImageConfig {
            entrypoint: Some(vec!["/docker-entrypoint.sh".into()]),
            cmd: Some(vec!["nginx".into(), "-g".into(), "daemon off;".into()]),
            env: Some(vec!["PATH=/usr/bin".into(), "NGINX_VERSION=1.27".into()]),
            exposed_ports: Some(vec!["443/tcp".into(), "80/tcp".into(), "bad".into()]),
            working_dir: Some("/usr/share/nginx".into()),
            user: Some("nginx".into()),
            labels: Some(HashMap::from([
                ("maintainer".to_string(), "NGINX".to_string()),
                ("a.first".to_string(), "1".to_string()),
            ])),
            ..ImageConfig::default()
        }),
        ..ImageInspect::default()
    }
}

#[test]
fn maps_the_inspect_result_and_an_empty_one_gives_defaults() {
    let detail = image_detail(nginx());
    assert_eq!(detail.id, "sha256:abc");
    assert_eq!(detail.repo_tags, ["nginx:1.27"]);
    assert_eq!(detail.repo_digests, ["nginx@sha256:def"]);
    assert_eq!(detail.created, "2024-05-01T12:30:45Z");
    assert_eq!(detail.platform(), "linux/arm64/v8");
    assert_eq!(detail.size, 1024);

    let config = &detail.config;
    assert_eq!(
        config.command_line(),
        "/docker-entrypoint.sh nginx -g daemon off;"
    );
    assert_eq!(config.env.len(), 2);
    assert_eq!(config.env[1].key, "NGINX_VERSION");
    let ports: Vec<String> = config.exposed_ports.iter().map(|p| p.to_string()).collect();
    assert_eq!(ports, ["80/tcp", "443/tcp"]);
    assert_eq!(config.working_dir, "/usr/share/nginx");
    assert_eq!(config.user, "nginx");
    assert_eq!(config.labels[0], ("a.first".into(), "1".into()));
    let detail = image_detail(ImageInspect {
        repo_tags: Some(vec!["<none>:<none>".into()]),
        size: Some(-1),
        ..ImageInspect::default()
    });
    assert!(detail.repo_tags.is_empty());
    assert_eq!(detail.size, 0);
    assert!(detail.config.exposed_ports.is_empty());
}

#[test]
fn maps_a_history_step_and_a_null_history_is_empty() {
    let layer = image_layer(ImageHistoryResponseItem {
        id: "<missing>".into(),
        created: 1_700_000_000,
        created_by: "/bin/sh -c #(nop)  CMD [\"sh\"]".into(),
        size: 4096,
        ..ImageHistoryResponseItem::default()
    });
    assert_eq!(layer.id, "<missing>");
    assert_eq!(layer.created, 1_700_000_000);
    assert_eq!(layer.size, 4096);
    assert_eq!(layer.command(), "CMD [\"sh\"]");
    let null = Error::JsonDataError {
        message: "invalid type: null, expected a sequence".into(),
        column: 4,
    };
    assert_eq!(image_history(Err(null)), Ok(Vec::new()));
    let other = Error::JsonDataError {
        message: "expected value".into(),
        column: 1,
    };
    assert!(image_history(Err(other)).is_err());
    let one = vec![ImageHistoryResponseItem::default()];
    assert_eq!(image_history(Ok(one)).map(|layers| layers.len()), Ok(1));
}

use super::{ImageConfig, ImageDetail};

#[test]
fn platform_skips_empty_parts() {
    let mut detail = ImageDetail {
        os: "linux".into(),
        architecture: "arm64".into(),
        variant: "v8".into(),
        ..ImageDetail::default()
    };
    assert_eq!(detail.platform(), "linux/arm64/v8");
    detail.variant.clear();
    assert_eq!(detail.platform(), "linux/arm64");
    assert_eq!(ImageDetail::default().platform(), "");
}

#[test]
fn created_label_shows_date_and_minutes() {
    let detail = |created: &str| ImageDetail {
        created: created.into(),
        ..ImageDetail::default()
    };
    assert_eq!(
        detail("2024-05-01T12:30:45.123Z").created_label(),
        "2024-05-01 12:30 UTC"
    );
    assert_eq!(detail("").created_label(), "—");
    assert_eq!(detail("yesterday").created_label(), "yesterday");
}

#[test]
fn command_line_joins_entrypoint_and_cmd() {
    let config = ImageConfig {
        entrypoint: vec!["/docker-entrypoint.sh".into()],
        cmd: vec!["nginx".into(), "-g".into(), "daemon off;".into()],
        ..ImageConfig::default()
    };
    assert_eq!(
        config.command_line(),
        "/docker-entrypoint.sh nginx -g daemon off;"
    );
    assert_eq!(ImageConfig::default().command_line(), "");
}

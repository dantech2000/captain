use super::ComposeLabels;

#[test]
fn split_config_files_drops_empty_parts() {
    assert_eq!(
        ComposeLabels::split_config_files("/a/compose.yaml, /a/override.yaml,,"),
        ["/a/compose.yaml", "/a/override.yaml"]
    );
    assert!(ComposeLabels::split_config_files("").is_empty());
}

use super::ComposeLabels;

#[test]
fn split_config_files_drops_only_captains_stdin_override() {
    let value = "/a/compose.yaml, /a/override.yaml,,-";
    assert_eq!(
        ComposeLabels::split_config_files(value, true),
        ["/a/compose.yaml", "/a/override.yaml"]
    );
    // Without Captain's label, `-` is the user's own file on stdin.
    assert_eq!(
        ComposeLabels::split_config_files(value, false),
        ["/a/compose.yaml", "/a/override.yaml", "-"]
    );
    assert!(ComposeLabels::split_config_files("", false).is_empty());
}

use super::list;
use crate::cancel::Cancel;

#[test]
fn a_cancelled_start_does_not_download_the_version_list() {
    let dir = std::env::temp_dir().join(format!("captain-k3s-list-{}", std::process::id()));
    let cancel = Cancel::default();
    cancel.cancel();
    let versions = list(
        &dir.join("versions.json"),
        &dir.join("cache"),
        true,
        &cancel,
    );
    assert!(versions.channels.is_empty());
    assert!(!dir.join("versions.json").exists());
    std::fs::remove_dir_all(&dir).ok();
}

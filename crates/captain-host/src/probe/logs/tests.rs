use super::log_bytes;

#[test]
fn sums_only_log_files() {
    let dir = std::env::temp_dir().join(format!("captain-log-bytes-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("ha.stderr.log"), [0u8; 300]).unwrap();
    std::fs::write(dir.join("serialv.log"), [0u8; 20]).unwrap();
    std::fs::write(dir.join("diffdisk"), [0u8; 1000]).unwrap();
    let total = log_bytes(&dir);
    std::fs::remove_dir_all(&dir).ok();
    assert_eq!(total, Some(320));
    assert_eq!(log_bytes(&dir), None);
}

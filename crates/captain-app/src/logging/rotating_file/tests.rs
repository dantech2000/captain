use std::io::Write;

use super::RotatingFile;

#[test]
fn rotates_to_one_old_file_at_the_limit() {
    let dir = std::env::temp_dir().join(format!("captain-rotating-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("captain.log");
    let mut file = RotatingFile::with_limit(path.clone(), 10).unwrap();
    for line in [b"aaaaaa\n", b"bbbbbb\n", b"cccccc\n"] {
        file.write_all(line).unwrap();
    }
    let current = std::fs::read_to_string(&path).unwrap();
    let old = std::fs::read_to_string(file.old_path()).unwrap();
    std::fs::remove_dir_all(&dir).ok();
    assert_eq!(current, "cccccc\n");
    assert_eq!(old, "bbbbbb\n");
}

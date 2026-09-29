use super::reserve;

#[test]
fn a_save_does_not_replace_a_dangling_link_or_a_taken_name() {
    let dir = std::env::temp_dir().join(format!("captain-reserve-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::os::unix::fs::symlink(dir.join("missing"), dir.join("app.tar")).unwrap();

    let (first, _) = reserve(&dir, "app.tar").unwrap();
    let (second, _) = reserve(&dir, "app.tar").unwrap();
    let link_kept = dir.join("app.tar").symlink_metadata().unwrap().is_symlink();
    std::fs::remove_dir_all(&dir).ok();

    assert_eq!(first, dir.join("app (1).tar"));
    assert_eq!(second, dir.join("app (2).tar"));
    assert!(link_kept);
}

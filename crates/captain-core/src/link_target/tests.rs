use super::link_target;

#[test]
fn follows_relative_links_and_keeps_plain_files() {
    let dir = std::env::temp_dir().join(format!("captain-link-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("dotfiles")).unwrap();
    let link = dir.join("config");
    std::os::unix::fs::symlink("dotfiles/config", &link).unwrap();
    assert_eq!(link_target(&link), dir.join("dotfiles/config"));
    let plain = dir.join("dotfiles/config");
    assert_eq!(link_target(&plain), plain);
    std::fs::remove_dir_all(&dir).ok();
}

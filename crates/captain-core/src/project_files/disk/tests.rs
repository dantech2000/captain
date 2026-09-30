use std::os::unix::fs::PermissionsExt;

use super::*;

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("captain-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn save_writes_through_a_symlink_and_keeps_permissions() {
    let dir = temp_dir("editor-save");
    let target = dir.join("real.yaml");
    std::fs::write(&target, "a: 1\n").unwrap();
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o640)).unwrap();
    let link = dir.join("compose.yaml");
    std::os::unix::fs::symlink("real.yaml", &link).unwrap();

    let loaded = read_text(&link).unwrap();
    let saved = save_text(&link, "a: 2\n", &loaded.version).unwrap();

    assert!(std::fs::symlink_metadata(&link).unwrap().is_symlink());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "a: 2\n");
    let mode = std::fs::metadata(&target).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o640);
    assert_eq!(disk_version(&link), Some(saved));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn save_refuses_when_the_file_changed_on_disk() {
    let dir = temp_dir("editor-changed");
    let file = dir.join("compose.yaml");
    std::fs::write(&file, "a: 1\n").unwrap();
    let loaded = read_text(&file).unwrap();
    std::fs::write(&file, "a: outside\n").unwrap();

    let result = save_text(&file, "a: mine\n", &loaded.version);

    assert!(matches!(result, Err(SaveError::Changed)));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "a: outside\n");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn save_refuses_a_change_made_while_it_writes() {
    let dir = temp_dir("editor-race");
    let file = dir.join("compose.yaml");
    std::fs::write(&file, "a: 1\n").unwrap();
    let loaded = read_text(&file).unwrap();

    let result = save_staged(&file, "a: mine\n", &loaded.version, || {
        std::fs::write(&file, "a: outside\n").unwrap();
    });

    assert!(matches!(result, Err(SaveError::Changed)));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "a: outside\n");
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
    std::fs::remove_dir_all(&dir).ok();
}

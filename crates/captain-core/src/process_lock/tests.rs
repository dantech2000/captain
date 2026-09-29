use super::{ProcessLock, app_lock_path};

#[test]
fn a_held_lock_refuses_a_second_holder_until_dropped() {
    let dir = std::env::temp_dir().join(format!("captain-lock-{}", std::process::id()));
    let path = dir.join("engine.lock");
    let first = ProcessLock::try_acquire(&path, "starting").expect("lock");
    assert!(first.is_some());
    assert!(
        ProcessLock::try_acquire(&path, "stopping")
            .expect("lock")
            .is_none()
    );
    let note = ProcessLock::holder(&path).expect("held");
    if cfg!(unix) {
        assert_eq!(note, "starting");
    }
    drop(first);
    assert_eq!(ProcessLock::holder(&path), None);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_app_lock_sits_next_to_the_settings() {
    let path = app_lock_path(std::path::Path::new("/cfg/Captain/settings.json"));
    assert_eq!(path, std::path::Path::new("/cfg/Captain/app.lock"));
}

#[cfg(unix)]
#[test]
fn a_lock_that_cannot_be_checked_counts_as_held() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("captain-lock-unreadable-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("app.lock");
    std::fs::write(&path, "").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
    let readable = std::fs::File::open(&path).is_ok();
    // Root reads any file, so the case cannot happen there.
    if !readable {
        assert_eq!(ProcessLock::holder(&path), Some(String::new()));
    }
    std::fs::remove_dir_all(&dir).ok();
}

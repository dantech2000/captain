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

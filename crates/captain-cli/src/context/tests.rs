use super::Context;
use captain_core::process_lock::{ProcessLock, app_lock_path};

#[test]
fn a_restore_keeps_the_app_out_but_not_its_own_settings_change() {
    let dir = std::env::temp_dir().join(format!("captain-cli-exclude-{}", std::process::id()));
    let context = Context::new(Some(dir.join("settings.json")), None, None).unwrap();
    let held = context.exclude_app("running").unwrap();
    assert!(!context.app_running());
    let app = ProcessLock::try_acquire(&app_lock_path(&context.settings_path), "running");
    assert!(app.unwrap().is_none());
    context.update_settings("running", |_| Ok(())).unwrap();
    drop(held);
    std::fs::remove_dir_all(&dir).ok();
}

use captain_core::GIB;
use captain_core::process_lock::{ProcessLock, app_lock_path};
use captain_core::settings::Settings;

use super::run;
use crate::context::Context;
use crate::settings_keys::{Machine, SettingKey};

fn context(name: &str) -> Context {
    let dir = std::env::temp_dir().join(format!("captain-cli-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    Context {
        settings_path: dir.join("settings.json"),
        machine: Machine {
            cpus: 8,
            memory_bytes: 16 * GIB,
            captain_available: true,
        },
        engine_paths: None,
    }
}

#[test]
fn set_writes_the_settings_file() {
    let context = context("write");
    run(&context, SettingKey::DebugLogging, "true").expect("set");
    let saved = Settings::load(&context.settings_path).expect("load");
    assert!(saved.debug_logging);
    std::fs::remove_dir_all(context.settings_path.parent().unwrap()).ok();
}

#[test]
fn set_refuses_while_the_app_runs() {
    let context = context("app");
    let _app = ProcessLock::try_acquire(&app_lock_path(&context.settings_path), "running")
        .expect("lock")
        .expect("free");
    assert!(run(&context, SettingKey::DebugLogging, "true").is_err());
    assert!(!context.settings_path.exists());
    std::fs::remove_dir_all(context.settings_path.parent().unwrap()).ok();
}

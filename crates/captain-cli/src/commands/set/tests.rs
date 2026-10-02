use captain_core::process_lock::{ProcessLock, app_lock_path, settings_lock_path};
use captain_core::settings::Settings;
use captain_core::{GIB, HostStatus};

use super::{disk_floor, run};
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
        holds_app: Default::default(),
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

#[test]
fn set_waits_for_another_writer_and_keeps_its_change() {
    let context = context("lock");
    let lock = ProcessLock::acquire(&settings_lock_path(&context.settings_path)).expect("lock");
    std::thread::scope(|scope| {
        let set = scope.spawn(|| run(&context, SettingKey::DebugLogging, "true"));
        std::thread::sleep(std::time::Duration::from_millis(200));
        assert!(!set.is_finished(), "set waits for the lock");
        let other = Settings {
            engine_endpoint: Some("unix:///tmp/other.sock".into()),
            ..Settings::default()
        };
        other.save(&context.settings_path).expect("save");
        drop(lock);
        set.join().unwrap().expect("set");
    });
    let saved = Settings::load(&context.settings_path).expect("load");
    assert!(saved.debug_logging);
    assert_eq!(
        saved.engine_endpoint.as_deref(),
        Some("unix:///tmp/other.sock")
    );
    std::fs::remove_dir_all(context.settings_path.parent().unwrap()).ok();
}

#[test]
fn set_disk_needs_a_readable_disk_unless_no_machine_exists() {
    assert_eq!(disk_floor(Some(HostStatus::NotCreated), None), Ok(None));
    assert_eq!(
        disk_floor(Some(HostStatus::Stopped), Some(GIB)),
        Ok(Some(GIB))
    );
    let failed = Some(HostStatus::Failed("limactl list failed".into()));
    assert!(disk_floor(failed, None).is_err());
    assert!(disk_floor(None, None).is_err());
}

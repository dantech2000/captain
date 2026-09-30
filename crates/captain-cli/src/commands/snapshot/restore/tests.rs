use captain_core::settings::Settings;

use super::run;
use crate::context::Context;

#[test]
fn a_broken_settings_file_stops_the_restore_before_the_snapshot_lookup() {
    let dir = std::env::temp_dir().join(format!("captain-cli-restore-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("settings.json"), "{").unwrap();
    let context = Context::new(
        Some(dir.join("settings.json")),
        Some(dir.join("lima")),
        None,
    )
    .unwrap();
    let host = context.host(&Settings::default());
    let snapshots = host.snapshots().expect("Lima snapshots");
    let error = run(&context, &host, &snapshots, "missing", true).unwrap_err();
    assert!(
        format!("{error:#}").contains("settings.json line 1:"),
        "{error:#}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

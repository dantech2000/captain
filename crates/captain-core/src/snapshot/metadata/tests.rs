use super::{Snapshot, SnapshotMetadata, find, newest_first};
use crate::daemon::DaemonSettings;
use crate::kubernetes::KubernetesSettings;
use crate::settings::Settings;
use crate::{GIB, HostResources};

fn snapshot(id: &str, name: &str, created: u64) -> Snapshot {
    Snapshot {
        id: id.into(),
        metadata: SnapshotMetadata {
            name: name.into(),
            description: String::new(),
            created,
            captain_version: "0.1.0".into(),
            lima_version: "2.2.0".into(),
            disk_allocated: 3 * GIB,
            resources: HostResources {
                cpus: 4,
                memory_bytes: 8 * GIB,
                disk_bytes: 64 * GIB,
            },
            daemon: None,
            kubernetes: None,
        },
    }
}

#[test]
fn metadata_round_trips_through_json_and_old_fields_default() {
    let metadata = snapshot("a", "base", 10).metadata;
    assert_eq!(
        SnapshotMetadata::from_json(&metadata.to_json()),
        Ok(metadata)
    );
    let minimal =
        r#"{"name":"x","created":1,"resources":{"cpus":2,"memory_bytes":1,"disk_bytes":1}}"#;
    let parsed = SnapshotMetadata::from_json(minimal).expect("parses");
    assert_eq!(parsed.description, "");
    assert_eq!(parsed.disk_allocated, 0);
}

#[test]
fn lists_newest_first_and_finds_by_name_before_id() {
    let mut snapshots = vec![snapshot("id-1", "old", 1), snapshot("old", "new", 2)];
    newest_first(&mut snapshots);
    assert_eq!(snapshots[0].metadata.name, "new");
    assert_eq!(find(&snapshots, "old").map(|s| s.id.as_str()), Some("id-1"));
    assert_eq!(
        find(&snapshots, "id-1").map(|s| s.id.as_str()),
        Some("id-1")
    );
    assert!(find(&snapshots, "missing").is_none());
}

#[test]
fn restoring_adopts_the_saved_engine_settings_and_keeps_them_when_absent() {
    let mut metadata = snapshot("a", "base", 10).metadata;
    let mut settings = Settings::default();
    settings.engine_daemon.tcp = true;
    metadata.adopt_into(&mut settings);
    assert_eq!(settings.engine_resources, Some(metadata.resources));
    assert!(
        settings.engine_daemon.tcp,
        "old snapshots keep the daemon settings"
    );

    metadata.daemon = Some(DaemonSettings::default());
    metadata.kubernetes = Some(KubernetesSettings {
        enabled: true,
        version: Some("v1.35.1+k3s1".into()),
        ..KubernetesSettings::default()
    });
    metadata.adopt_into(&mut settings);
    assert!(!settings.engine_daemon.tcp);
    assert_eq!(settings.kubernetes.version.as_deref(), Some("v1.35.1+k3s1"));
    assert_eq!(
        SnapshotMetadata::from_json(&metadata.to_json()),
        Ok(metadata)
    );
}

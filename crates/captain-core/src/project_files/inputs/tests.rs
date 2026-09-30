use super::*;

#[test]
fn a_changed_env_file_changes_the_versions() {
    let dir = std::env::temp_dir().join(format!("captain-inputs-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("compose.yaml"), "services: {}\n").unwrap();
    let project = ComposeProject {
        name: "shop".into(),
        working_dir: Some(dir.display().to_string()),
        config_files: vec!["compose.yaml".into()],
        services: Vec::new(),
    };

    let before = InputVersions::read(&project, &[]);
    assert!(!before.changed());
    std::fs::write(dir.join(".env"), "TAG=2\n").unwrap();

    assert!(before.changed());
    std::fs::remove_dir_all(&dir).ok();
}

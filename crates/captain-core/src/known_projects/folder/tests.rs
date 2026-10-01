use super::*;

#[test]
fn compose_files_follow_compose_order_and_add_the_first_override() {
    let dir = std::env::temp_dir().join(format!("captain-compose-files-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    assert!(compose_files_in(&dir).is_empty());
    for name in [
        "docker-compose.yml",
        "compose.yml",
        "docker-compose.override.yml",
        "compose.override.yaml",
    ] {
        std::fs::write(dir.join(name), "services: {}\n").unwrap();
    }

    assert_eq!(
        compose_files_in(&dir),
        vec![dir.join("compose.yml"), dir.join("compose.override.yaml")]
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_project_name_comes_from_config_json() {
    assert_eq!(
        project_name_from_config(r#"{"name":"shop","services":{}}"#),
        Ok("shop".into())
    );
    assert!(project_name_from_config(r#"{"services":{}}"#).is_err());
}

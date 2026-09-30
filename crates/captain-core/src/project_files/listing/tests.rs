use super::*;

/// Trimmed from `docker compose config --format json` of a project with two
/// services that build from one folder, one remote build, and one image.
const CONFIG: &str = r#"{
  "name": "shop",
  "services": {
    "api": {"build": {"context": "/srv/shop/app", "dockerfile": "Dockerfile"}},
    "worker": {"build": {"context": "/srv/shop/app", "dockerfile": "Dockerfile"}},
    "docs": {"build": {"context": "https://github.com/acme/docs.git"}},
    "outside": {"build": {"context": "/srv/other", "dockerfile": "Dockerfile"}},
    "db": {"image": "postgres:17"}
  }
}"#;

#[test]
fn dockerfiles_group_services_and_skip_remote_and_outside_contexts() {
    let dir = Path::new("/srv/shop");
    let files = dockerfiles(CONFIG, dir).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, dir.join("app").join("Dockerfile"));
    assert_eq!(files[0].services, ["api", "worker"]);
    assert_eq!(files[0].context.as_deref(), Some(dir.join("app").as_path()));
}

#[test]
fn compose_files_keep_only_files_inside_the_working_folder() {
    let project = ComposeProject {
        name: "shop".into(),
        working_dir: Some("/srv/shop".into()),
        config_files: vec!["compose.yaml".into(), "../shared/base.yaml".into()],
        services: Vec::new(),
    };
    let files = compose_files(&project);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, Path::new("/srv/shop").join("compose.yaml"));
}

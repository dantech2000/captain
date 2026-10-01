use std::path::PathBuf;

use super::*;

fn known(name: &str, dir: &str) -> KnownProject {
    KnownProject {
        name: name.into(),
        dir: PathBuf::from(dir),
        files: vec!["compose.yaml".into()],
        added: 0,
    }
}

fn live(name: &str, dir: &str) -> ComposeProject {
    ComposeProject {
        name: name.into(),
        working_dir: Some(dir.into()),
        // Joined as the code joins it, so the separator matches on Windows too.
        config_files: vec![
            PathBuf::from(dir)
                .join("compose.yaml")
                .display()
                .to_string(),
        ],
        services: Vec::new(),
    }
}

#[test]
fn stopped_known_leaves_out_names_that_run_from_any_folder() {
    let known = [
        known("web", "/code/web"),
        known("api", "/code/api"),
        known("db", "/code/db"),
    ];
    let running = [live("api", "/code/api"), live("db", "/elsewhere/db")];

    let stopped = stopped_known(&running, &known);

    assert_eq!(stopped, vec![live("web", "/code/web")]);
}

#[test]
fn a_running_project_is_known_only_with_the_same_name_and_folder() {
    let known = [known("api", "/code/api")];

    assert!(known_match(&live("api", "/code/api/"), &known).is_some());
    assert!(known_match(&live("api", "/other/api"), &known).is_none());
}

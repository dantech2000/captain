use std::path::{Path, PathBuf};

use captain_core::model::{ComposeProject, ProjectAction};

use super::compose_command;

fn project(files: &[&str]) -> ComposeProject {
    ComposeProject {
        name: "shop".into(),
        working_dir: Some("/code/shop".into()),
        config_files: files.iter().map(|f| f.to_string()).collect(),
        services: Vec::new(),
    }
}

fn args(command: &super::ComposeCommand) -> Vec<&str> {
    command.args.iter().map(String::as_str).collect()
}

const TMP: &str = "/tmp";

#[test]
fn up_passes_the_project_name_and_every_file() {
    let project = project(&["/code/shop/compose.yaml", "compose.override.yaml"]);
    let command =
        compose_command(&project, ProjectAction::Up, Path::new(TMP), |_| true).expect("command");

    let file = |name: &str| Path::new("/code/shop").join(name).display().to_string();
    let expected = [
        "compose",
        "--ansi",
        "never",
        "-p",
        "shop",
        "-f",
        "/code/shop/compose.yaml",
        "-f",
        &file("compose.override.yaml"),
        "up",
        "-d",
    ];
    assert_eq!(args(&command), expected);
    assert_eq!(command.dir, PathBuf::from("/code/shop"));
}

#[test]
fn up_fails_when_a_file_is_missing() {
    let project = project(&["/code/shop/compose.yaml"]);
    let error = compose_command(&project, ProjectAction::Up, Path::new(TMP), |p| {
        p == Path::new("/code/shop")
    })
    .expect_err("missing file");
    assert!(error.contains("cannot find"), "{error}");
}

#[test]
fn pull_fails_without_known_files() {
    let error = compose_command(&project(&[]), ProjectAction::Pull, Path::new(TMP), |_| true)
        .expect_err("no files");
    assert!(error.contains("does not know"), "{error}");
}

#[test]
fn down_uses_only_the_name_when_files_are_gone() {
    let project = project(&["/code/shop/compose.yaml"]);
    let command =
        compose_command(&project, ProjectAction::Down, Path::new(TMP), |_| false).expect("command");
    assert_eq!(
        args(&command),
        ["compose", "--ansi", "never", "-p", "shop", "down"]
    );
    assert_eq!(command.dir, PathBuf::from(TMP));
}

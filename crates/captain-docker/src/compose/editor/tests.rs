use std::path::Path;

use captain_core::model::ComposeProject;

use super::check_args;

#[test]
fn check_args_put_stdin_in_the_place_of_the_edited_file() {
    let project = ComposeProject {
        name: "shop".into(),
        working_dir: Some("/code/shop".into()),
        config_files: vec!["compose.yaml".into(), "compose.override.yaml".into()],
        services: Vec::new(),
    };
    let dir = Path::new("/code/shop");
    let args = check_args(&project, dir, &dir.join("compose.override.yaml"));
    let base = dir.join("compose.yaml").display().to_string();
    let dir = dir.display().to_string();
    let expected = [
        "compose",
        "--ansi",
        "never",
        "-p",
        "shop",
        "--project-directory",
        &dir,
        "-f",
        &base,
        "-f",
        "-",
        "config",
        "--quiet",
    ];
    assert_eq!(args, expected);
}

use std::path::PathBuf;
use std::process::Command;

use super::{ClientStep, line_diff, run_step};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("captain-run-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg(unix)]
#[test]
fn a_command_runs_with_its_arguments_as_shown() {
    use std::os::unix::fs::PermissionsExt;

    use super::shell_line;
    let dir = scratch("stub");
    let stub = dir.join("claude");
    let record = dir.join("args");
    std::fs::write(
        &stub,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n",
            record.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    let argv: Vec<String> = ["claude", "mcp", "add", "{\"a\": 1}"]
        .map(String::from)
        .to_vec();
    assert_eq!(shell_line(&argv), "claude mcp add '{\"a\": 1}'");
    let launch = |argv: &[String]| {
        let mut command = Command::new(dir.join(&argv[0]));
        command.args(&argv[1..]);
        command
    };
    run_step(&ClientStep::Run(argv), &launch).unwrap();
    assert_eq!(
        std::fs::read_to_string(&record).unwrap(),
        "mcp\nadd\n{\"a\": 1}\n"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn an_edit_refuses_a_file_that_changed_and_keeps_a_backup() {
    let dir = scratch("edit");
    let path = dir.join("mcp.json");
    std::fs::write(&path, "{}\n").unwrap();
    let step = |before: &str| ClientStep::Edit {
        path: path.clone(),
        before: before.into(),
        after: "{ \"servers\": {} }\n".into(),
    };
    let launch = |_: &[String]| Command::new("false");
    assert!(run_step(&step("{ }\n"), &launch).is_err());
    run_step(&step("{}\n"), &launch).unwrap();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "{ \"servers\": {} }\n"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("mcp.json.captain-backup")).unwrap(),
        "{}\n"
    );
    assert_eq!(line_diff("a\nb\nc", "a\nx\nc"), "- b\n+ x");
    std::fs::remove_dir_all(&dir).ok();
}

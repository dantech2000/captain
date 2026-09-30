use super::{ProjectTasks, TaskCommand};

#[test]
fn reads_string_and_list_commands_and_reports_entries_without_a_service() {
    let config = r#"{
        "services": {"api": {}},
        "x-captain": {"tasks": {
            "seed": {"service": "api", "command": ["node", "seed.js"]},
            "migrate": {"service": "api", "command": "npm run migrate"},
            "psql": {"command": "psql"}
        }}
    }"#;
    let tasks = ProjectTasks::parse(config).unwrap();
    let names: Vec<&str> = tasks.tasks.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["migrate", "seed"]);
    assert_eq!(
        tasks.tasks[0].command.exec_args(),
        ["sh", "-c", "npm run migrate"]
    );
    assert_eq!(
        tasks.tasks[1].command,
        TaskCommand::Args(vec!["node".into(), "seed.js".into()])
    );
    assert_eq!(tasks.problems, ["The task psql needs a service."]);
}

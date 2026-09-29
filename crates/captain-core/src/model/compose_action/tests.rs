use super::ProjectAction;

#[test]
fn up_runs_detached() {
    assert_eq!(ProjectAction::Up.args(), ["up", "-d"]);
    assert_eq!(ProjectAction::Down.args(), ["down"]);
}

#[test]
fn only_up_and_pull_need_the_files() {
    assert!(ProjectAction::Up.needs_files());
    assert!(ProjectAction::Pull.needs_files());
    assert!(!ProjectAction::Down.needs_files());
    assert!(!ProjectAction::Restart.needs_files());
}

#[test]
fn done_message_names_the_project() {
    assert_eq!(
        ProjectAction::Down.done_message("shop"),
        "Removed the containers of shop."
    );
}

use super::ContainerAction;
use crate::model::ContainerState;

#[test]
fn toggle_stops_live_containers_and_starts_the_rest() {
    assert_eq!(
        ContainerAction::toggle_for(ContainerState::Running),
        ContainerAction::Stop
    );
    assert_eq!(
        ContainerAction::toggle_for(ContainerState::Paused),
        ContainerAction::Stop
    );
    assert_eq!(
        ContainerAction::toggle_for(ContainerState::Exited),
        ContainerAction::Start
    );
}

#[test]
fn pause_toggle_depends_on_the_state() {
    assert_eq!(
        ContainerAction::pause_toggle_for(ContainerState::Running),
        Some(ContainerAction::Pause)
    );
    assert_eq!(
        ContainerAction::pause_toggle_for(ContainerState::Paused),
        Some(ContainerAction::Unpause)
    );
    assert_eq!(
        ContainerAction::pause_toggle_for(ContainerState::Exited),
        None
    );
    assert_eq!(
        ContainerAction::pause_toggle_for(ContainerState::Restarting),
        None
    );
}

#[test]
fn live_containers_need_a_force_remove() {
    assert_eq!(
        ContainerAction::removal_for(ContainerState::Running),
        ContainerAction::ForceRemove
    );
    assert_eq!(
        ContainerAction::removal_for(ContainerState::Paused),
        ContainerAction::ForceRemove
    );
    assert_eq!(
        ContainerAction::removal_for(ContainerState::Exited),
        ContainerAction::Remove
    );
    assert!(ContainerAction::Remove.removes());
    assert!(ContainerAction::ForceRemove.removes());
    assert!(!ContainerAction::Stop.removes());
}

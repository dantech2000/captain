use super::HostStatus;

#[test]
fn start_and_stop_follow_the_state() {
    assert!(HostStatus::Stopped.can_start());
    assert!(HostStatus::NotCreated.can_start());
    assert!(HostStatus::Failed("broken".into()).can_start());
    assert!(!HostStatus::Running.can_start());
    assert!(!HostStatus::NotInstalled("no lima".into()).can_start());

    assert!(HostStatus::Running.can_stop());
    assert!(HostStatus::Starting.can_stop());
    assert!(!HostStatus::Stopped.can_stop());
}

#[test]
fn busy_while_changing() {
    assert!(HostStatus::Starting.is_busy());
    assert!(HostStatus::Stopping.is_busy());
    assert!(!HostStatus::Running.is_busy());
}

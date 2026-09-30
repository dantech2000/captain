use super::{Readiness, readiness};
use crate::agent_tools::test_fleet::fleet;
use crate::model::{ContainerState, Health};

#[test]
fn waits_for_health_and_gives_up_on_a_stopped_container() {
    let mut containers = fleet(2);
    assert_eq!(
        readiness(&containers.iter().collect::<Vec<_>>()),
        Readiness::Ready
    );
    containers[0].health = Some(Health::Starting);
    let Readiness::Waiting(why) = readiness(&containers.iter().collect::<Vec<_>>()) else {
        panic!("should wait");
    };
    assert!(why.contains("project0-web-1 (running, starting)"), "{why}");
    containers[0].health = Some(Health::Healthy);
    containers[1].state = ContainerState::Exited;
    containers[1].status = "Exited (0) 5 seconds ago".into();
    assert_eq!(
        readiness(&containers.iter().collect::<Vec<_>>()),
        Readiness::Ready
    );
    containers[1].status = "Exited (1) 5 seconds ago".into();
    assert!(matches!(
        readiness(&containers.iter().collect::<Vec<_>>()),
        Readiness::Stopped(_)
    ));
}

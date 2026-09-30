use captain_core::HostStatus;

use super::{Trigger, probes_machine};

fn trigger(connection: u8, captain: Option<HostStatus>) -> Trigger {
    Trigger {
        connection,
        captain,
    }
}

#[test]
fn a_new_engine_status_probes_the_machine_but_a_new_connection_does_not() {
    let setting_up = trigger(0, Some(HostStatus::Starting));
    let running = trigger(1, Some(HostStatus::Running));
    let reconnected = trigger(0, Some(HostStatus::Running));

    assert!(probes_machine(None, Some(&setting_up)));
    assert!(probes_machine(Some(&setting_up), Some(&running)));
    assert!(!probes_machine(Some(&running), Some(&reconnected)));
}

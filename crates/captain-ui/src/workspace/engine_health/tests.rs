use captain_core::{EngineError, HostStatus};

use super::EngineHealth;
use crate::workspace::Connection;

#[test]
fn a_drop_shows_reconnecting_then_not_answering() {
    let running = Some(&HostStatus::Running);
    let failed = Connection::Failed(EngineError::Unreachable("gone".into()));
    for host in [running, None] {
        let of = |connection, retrying| EngineHealth::of(host, connection, retrying);
        assert_eq!(of(&Connection::Connecting, false), EngineHealth::Connecting);
        assert_eq!(
            of(&Connection::Connecting, true),
            EngineHealth::Reconnecting
        );
        assert_eq!(of(&failed, true), EngineHealth::NotAnswering);
    }
    // A stopped host decides, whatever the connection says.
    let stopped = Some(&HostStatus::Stopped);
    assert_eq!(
        EngineHealth::of(stopped, &Connection::Connecting, true),
        EngineHealth::Stopped
    );
}

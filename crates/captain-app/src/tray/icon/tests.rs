use super::{SIZE, TURN_FRAMES, rgba};
use crate::tray::snapshot::EngineStatus;

#[test]
fn each_engine_state_draws_a_different_icon() {
    let states = [
        EngineStatus::Stopped,
        EngineStatus::Starting,
        EngineStatus::Running,
        EngineStatus::NeedsAttention,
    ];
    let icons: Vec<Vec<u8>> = states.iter().map(|&s| rgba(s, 1, [0, 0, 0])).collect();
    for (i, icon) in icons.iter().enumerate() {
        assert_eq!(icon.len(), (SIZE * SIZE * 4) as usize);
        for other in &icons[i + 1..] {
            assert_ne!(icon, other, "{:?}", states[i]);
        }
    }
}

#[test]
fn the_wheel_turns_while_starting_and_loops() {
    let frame = |n| rgba(EngineStatus::Starting, n, [0, 0, 0]);
    assert_ne!(frame(0), frame(1));
    assert_eq!(frame(0), frame(TURN_FRAMES));
    assert_eq!(
        rgba(EngineStatus::Running, 0, [0; 3]),
        rgba(EngineStatus::Running, 3, [0; 3])
    );
}

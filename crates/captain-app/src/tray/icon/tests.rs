use super::{SIZE, rgba};
use crate::tray::snapshot::EngineStatus;

#[test]
fn each_engine_state_draws_a_different_icon() {
    let states = [
        EngineStatus::Stopped,
        EngineStatus::Starting,
        EngineStatus::Running,
        EngineStatus::NeedsAttention,
    ];
    let icons: Vec<Vec<u8>> = states.iter().map(|&s| rgba(s, [0, 0, 0])).collect();
    for (i, icon) in icons.iter().enumerate() {
        assert_eq!(icon.len(), (SIZE * SIZE * 4) as usize);
        for other in &icons[i + 1..] {
            assert_ne!(icon, other, "{:?}", states[i]);
        }
    }
}

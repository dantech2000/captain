use super::stops_engine;

#[test]
fn quit_stops_a_captain_start_after_a_switch_to_another_engine() {
    assert!(stops_engine(true, false, false));
    assert!(!stops_engine(false, false, true));
    assert!(stops_engine(false, true, true));
}

use super::*;

#[test]
fn the_suggestion_skips_published_and_busy_ports() {
    assert_eq!(suggest_port(5432, &[], |_| true), 5432);
    assert_eq!(suggest_port(5432, &[5432, 5433], |port| port != 5434), 5435);
    assert_eq!(suggest_port(80, &[], |_| false), 80);
}

use super::Health;

#[test]
fn parses_health_states() {
    assert_eq!(Health::parse("healthy"), Some(Health::Healthy));
    assert_eq!(Health::parse("starting"), Some(Health::Starting));
    assert_eq!(Health::parse("unhealthy"), Some(Health::Unhealthy));
    assert_eq!(Health::parse("none"), None);
    assert_eq!(Health::parse(""), None);
}
